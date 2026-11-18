use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use dever_core::game::GameError;
use serde_json::json;

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl ApiError {
    pub fn bad(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: message.into(),
        }
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({ "error": self.message }))).into_response()
    }
}

/// The screens speak Portuguese; the game's errors do not.
impl From<GameError> for ApiError {
    fn from(e: GameError) -> Self {
        let message = match &e {
            GameError::UnknownPlayer => "jogador desconhecido".to_owned(),
            GameError::WrongPhase => "isso não é possível nesta fase".to_owned(),
            GameError::NameTaken => "esse nome já está em uso".to_owned(),
            GameError::EmptyName => "escolha um nome".to_owned(),
            GameError::NotRegistered => "cadastre a vitrine primeiro".to_owned(),
            GameError::AlreadyRegistered => "a vitrine já está cadastrada".to_owned(),
            GameError::Suspended => {
                "inscrição estadual suspensa: pague o auto de infração pra operar".to_owned()
            }
            GameError::Locked => "isso ainda não foi liberado".to_owned(),
            GameError::NeedsTaproot => {
                "Silent Payments precisam de uma carteira Taproot".to_owned()
            }
            GameError::UnknownInvoice => "pedido não encontrado".to_owned(),
            GameError::InvoiceClosed => "esse pedido não está mais aberto".to_owned(),
            GameError::SelfPayment => "você não pode pagar a si mesmo".to_owned(),
            GameError::NotInCatalog(item) => format!("{item} não está no seu catálogo"),
            GameError::NoDemand => "ninguém compra do seu setor nesta rodada".to_owned(),
            GameError::InsufficientFunds => {
                "moedas insuficientes (as congeladas não contam)".to_owned()
            }
            GameError::NotYourCoin(_) => "essa moeda não é sua ou já foi gasta".to_owned(),
            GameError::Frozen => "essa moeda está congelada".to_owned(),
            GameError::WrongCoin => "essa moeda não serve pra essa ação".to_owned(),
            GameError::UnknownAuto => "auto de infração não encontrado".to_owned(),
            GameError::AutoPaid => "esse auto já foi pago".to_owned(),
            GameError::RoomClosed => "você já está na mesa, ou a mesa fechou".to_owned(),
            GameError::Ledger(inner) => format!("ledger: {inner}"),
            GameError::Economy(inner) => format!("economia: {inner}"),
        };
        let status = match e {
            GameError::UnknownPlayer => StatusCode::UNAUTHORIZED,
            GameError::UnknownInvoice | GameError::UnknownAuto => StatusCode::NOT_FOUND,
            GameError::Suspended | GameError::Locked | GameError::NotRegistered => {
                StatusCode::FORBIDDEN
            }
            GameError::WrongPhase | GameError::InvoiceClosed | GameError::RoomClosed => {
                StatusCode::CONFLICT
            }
            _ => StatusCode::BAD_REQUEST,
        };
        Self { status, message }
    }
}
