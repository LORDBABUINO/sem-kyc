//! Invoices shown as QR codes, payments with coin selection, Payjoin and the
//! cooperative's bonus.

use bitcoin::address::NetworkUnchecked;
use bitcoin::hashes::{sha256, Hash};
use bitcoin::{Address, Amount, OutPoint, Txid};
use serde::Serialize;

use super::{Event, Game, GameError, ItemStatus, Operation, ReceiveMode, Unlock};
use crate::address::ScriptKind;
use crate::economy::{self, Item};
use crate::ledger::{Origin, OutputSpec, Owner, PlayerId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum InvoiceStatus {
    Open,
    PayjoinPending {
        buyer: PlayerId,
        #[serde(skip)]
        inputs: Vec<OutPoint>,
    },
    Paid {
        txid: Txid,
        buyer: PlayerId,
    },
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Invoice {
    pub code: String,
    pub seller: PlayerId,
    pub item: Item,
    pub amount: Amount,
    #[serde(with = "stealth_model::types::serde_addr")]
    pub address: Address<NetworkUnchecked>,
    /// What the QR shows: the address, or the sp1 string.
    pub display: String,
    pub payjoin: bool,
    pub round: u8,
    pub status: InvoiceStatus,
    pub created: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "result")]
pub enum Payment {
    Settled { txid: Txid },
    AwaitingSeller,
}

const CODE_CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

impl Game {
    /// The seller's price for this round: equal revenue per seller, in the
    /// seller's price style.
    pub fn unit_price(&self, seller: PlayerId, round: u8) -> Result<Amount, GameError> {
        let s = self.player(seller)?;
        let buyers = self
            .players
            .values()
            .filter(|p| {
                self.trails
                    .mission(p.sector, round)
                    .iter()
                    .any(|i| i.supplier == s.sector)
            })
            .count();
        let sellers = self
            .players
            .values()
            .filter(|p| p.sector == s.sector)
            .count();
        let salt = round as u64 * 16 + s.sector as u64;
        Ok(economy::unit_price(
            self.config.target_revenue,
            buyers,
            sellers,
            s.price_style,
            salt,
        ))
    }

    pub fn invoice(&self, code: &str) -> Option<&Invoice> {
        self.invoices.get(&code.to_ascii_uppercase())
    }

    pub fn invoices_paid_by_txid(&self, txid: Txid) -> Option<&Invoice> {
        self.invoices
            .values()
            .find(|i| matches!(i.status, InvoiceStatus::Paid { txid: t, .. } if t == txid))
    }

    pub fn invoices_of(&self, seller: PlayerId) -> Vec<&Invoice> {
        let mut list: Vec<&Invoice> = self
            .invoices
            .values()
            .filter(|i| i.seller == seller)
            .collect();
        list.sort_by_key(|i| i.created);
        list
    }

    fn next_code(&mut self) -> String {
        loop {
            let mut data = self.config.seed.to_vec();
            data.extend_from_slice(&self.next_code.to_le_bytes());
            self.next_code += 1;
            let hash = sha256::Hash::hash(&data).to_byte_array();
            let code: String = hash[..6]
                .iter()
                .map(|b| CODE_CHARS[(*b % 32) as usize] as char)
                .collect();
            if !self.invoices.contains_key(&code) {
                return code;
            }
        }
    }

    /// The seller picks the item the buyer asked for; the QR comes out of here.
    pub fn create_invoice(
        &mut self,
        seller: PlayerId,
        item_name: &str,
        now: u64,
    ) -> Result<Invoice, GameError> {
        let round = self.negotiation_round()?;
        let p = self.require_active(seller)?;
        let sector = p.sector;
        let mode = p.receive_mode;
        let silent = p.silent_address.clone();
        let storefront = p.storefront.clone();
        let item = self
            .trails
            .catalog(sector)
            .into_iter()
            .find(|i| i.name == item_name)
            .cloned()
            .ok_or_else(|| GameError::NotInCatalog(item_name.to_owned()))?;
        let amount = self.unit_price(seller, round)?;
        let (address, display, payjoin) = match mode {
            ReceiveMode::Storefront => {
                let d = storefront.assume_checked_ref().to_string();
                (storefront, d, false)
            }
            ReceiveMode::Fresh => {
                let a = self.fresh_for(seller)?;
                let d = a.assume_checked_ref().to_string();
                (a, d, false)
            }
            ReceiveMode::Payjoin => {
                let a = self.fresh_for(seller)?;
                let d = a.assume_checked_ref().to_string();
                (a, d, true)
            }
            ReceiveMode::SilentPayments => {
                let a = self.book.fresh(ScriptKind::Taproot);
                let d = silent.ok_or(GameError::NeedsTaproot)?;
                (a, d, false)
            }
        };
        let code = self.next_code();
        let invoice = Invoice {
            code: code.clone(),
            seller,
            item,
            amount,
            address,
            display,
            payjoin,
            round,
            status: InvoiceStatus::Open,
            created: now,
        };
        self.invoices.insert(code, invoice.clone());
        Ok(invoice)
    }

    /// The buyer scanned the QR. With `payjoin` the seller still has to answer.
    pub fn pay_invoice(
        &mut self,
        buyer: PlayerId,
        code: &str,
        chosen: Option<&[OutPoint]>,
        payjoin: bool,
    ) -> Result<Payment, GameError> {
        self.negotiation_round()?;
        self.require_active(buyer)?;
        let code = code.to_ascii_uppercase();
        let invoice = self
            .invoices
            .get(&code)
            .ok_or(GameError::UnknownInvoice)?
            .clone();
        if invoice.status != InvoiceStatus::Open {
            return Err(GameError::InvoiceClosed);
        }
        if invoice.seller == buyer {
            return Err(GameError::SelfPayment);
        }
        let inputs = self.select_coins(buyer, invoice.amount, chosen)?;
        if payjoin && invoice.payjoin {
            if !self.player(buyer)?.has(Unlock::Payjoin) {
                return Err(GameError::Locked);
            }
            if let Some(i) = self.invoices.get_mut(&code) {
                i.status = InvoiceStatus::PayjoinPending { buyer, inputs };
            }
            self.events.push(Event::PayjoinRequested {
                code,
                buyer,
                seller: invoice.seller,
            });
            return Ok(Payment::AwaitingSeller);
        }
        let txid = self.settle(buyer, &code, inputs, None)?;
        Ok(Payment::Settled { txid })
    }

    /// The seller answers a Payjoin proposal: a coin to contribute, or none.
    pub fn respond_payjoin(
        &mut self,
        seller: PlayerId,
        code: &str,
        contribute: Option<OutPoint>,
    ) -> Result<Txid, GameError> {
        let code = code.to_ascii_uppercase();
        let invoice = self
            .invoices
            .get(&code)
            .ok_or(GameError::UnknownInvoice)?
            .clone();
        if invoice.seller != seller {
            return Err(GameError::UnknownInvoice);
        }
        let InvoiceStatus::PayjoinPending { buyer, inputs } = invoice.status else {
            return Err(GameError::InvoiceClosed);
        };
        // The buyer may have spent those coins meanwhile: check them again,
        // and reopen the invoice if they are gone.
        if let Err(e) = self.select_coins(buyer, invoice.amount, Some(&inputs)) {
            if let Some(i) = self.invoices.get_mut(&code) {
                i.status = InvoiceStatus::Open;
            }
            return Err(e);
        }
        if let Some(op) = contribute {
            match self.ledger.coin(&op) {
                Some(c) if c.owner == Owner::Player(seller) && !c.frozen => {}
                Some(c) if c.owner == Owner::Player(seller) => return Err(GameError::Frozen),
                _ => return Err(GameError::NotYourCoin(op)),
            }
        }
        self.settle(buyer, &code, inputs, contribute)
    }

    fn settle(
        &mut self,
        buyer: PlayerId,
        code: &str,
        inputs: Vec<OutPoint>,
        seller_input: Option<OutPoint>,
    ) -> Result<Txid, GameError> {
        let invoice = self
            .invoices
            .get(code)
            .ok_or(GameError::UnknownInvoice)?
            .clone();
        let seller = invoice.seller;
        let paid: Amount = inputs
            .iter()
            .filter_map(|op| self.ledger.coin(op))
            .map(|c| c.amount)
            .sum();
        let contributed = seller_input
            .and_then(|op| self.ledger.coin(&op))
            .map(|c| c.amount)
            .unwrap_or(Amount::ZERO);
        let mut all_inputs = inputs.clone();
        all_inputs.extend(seller_input);
        let mut outputs = vec![OutputSpec {
            address: invoice.address.clone(),
            amount: invoice.amount + contributed,
            owner: Owner::Player(seller),
            origin: Origin::Sale(buyer),
        }];
        let change = paid - invoice.amount;
        if change > Amount::ZERO {
            outputs.push(OutputSpec {
                address: self.fresh_for(buyer)?,
                amount: change,
                owner: Owner::Player(buyer),
                origin: Origin::Change,
            });
        }
        let txid = self.ledger.spend(&all_inputs, outputs)?;
        self.observe(txid, vec![buyer, seller]);
        if let Some(i) = self.invoices.get_mut(code) {
            i.status = InvoiceStatus::Paid { txid, buyer };
        }
        self.operations.push(Operation {
            round: invoice.round,
            txid,
            payer: buyer,
            payee: seller,
            amount: invoice.amount,
            vout: 0,
        });
        self.events.push(Event::InvoicePaid {
            code: code.to_owned(),
            buyer,
            seller,
            txid,
        });
        let seller_sector = self.player(seller)?.sector;
        if let Some(m) = self
            .players
            .get_mut(&buyer)
            .and_then(|p| p.missions.get_mut(&invoice.round))
        {
            if let Some(mi) = m.items.iter_mut().find(|mi| {
                mi.status == ItemStatus::Pending
                    && mi.item.name == invoice.item.name
                    && mi.item.supplier == seller_sector
            }) {
                mi.status = ItemStatus::Paid { txid };
            }
        }
        self.maybe_bonus(buyer, invoice.round)?;
        Ok(txid)
    }

    fn maybe_bonus(&mut self, buyer: PlayerId, round: u8) -> Result<(), GameError> {
        let Some(mission) = self.player(buyer)?.missions.get(&round) else {
            return Ok(());
        };
        if !mission.complete() || mission.bonus.is_some() {
            return Ok(());
        }
        let spent: Amount = self
            .operations
            .iter()
            .filter(|o| o.payer == buyer && o.round == round)
            .map(|o| o.amount)
            .sum();
        let amount = Amount::from_sat(spent.to_sat() * self.config.bonus_percent / 100);
        if amount == Amount::ZERO {
            return Ok(());
        }
        let mut coins = self.ledger.coins_of(Owner::Cooperative);
        coins.sort_by_key(|c| std::cmp::Reverse(c.amount));
        let coin = coins
            .first()
            .filter(|c| c.amount >= amount)
            .ok_or(GameError::InsufficientFunds)?;
        let (op, total) = (coin.outpoint, coin.amount);
        let to = self.fresh_for(buyer)?;
        let mut outputs = vec![OutputSpec {
            address: to,
            amount,
            owner: Owner::Player(buyer),
            origin: Origin::Bonus,
        }];
        if total > amount {
            outputs.push(OutputSpec {
                address: self.book.fresh(ScriptKind::Segwit),
                amount: total - amount,
                owner: Owner::Cooperative,
                origin: Origin::Change,
            });
        }
        let txid = self.ledger.spend(&[op], outputs)?;
        self.observe(txid, vec![buyer]);
        if let Some(m) = self
            .players
            .get_mut(&buyer)
            .and_then(|p| p.missions.get_mut(&round))
        {
            m.bonus = Some(txid);
        }
        self.events.push(Event::Bonus {
            player: buyer,
            txid,
            amount,
        });
        Ok(())
    }

    pub(super) fn expire_invoices(&mut self, round: u8) {
        for i in self.invoices.values_mut() {
            if i.round == round
                && matches!(
                    i.status,
                    InvoiceStatus::Open | InvoiceStatus::PayjoinPending { .. }
                )
            {
                i.status = InvoiceStatus::Expired;
            }
        }
    }
}
