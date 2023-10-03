use {crate::SWITCHBOARD_FUNCTION_SLOT_UNTIL_EXPIRATION, anchor_lang::prelude::*};

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct SwitchboardRequestInfo {
    pub account: Pubkey,
    pub status: SwitchboardFunctionRequestStatus,
}

impl SwitchboardRequestInfo {
    pub fn is_requested(&self) -> bool {
        matches!(
            self.status,
            SwitchboardFunctionRequestStatus::Requested { slot: _ }
        )
    }

    pub fn request_is_expired(&self, current_slot: u64) -> bool {
        matches!(self.status, SwitchboardFunctionRequestStatus::Requested { slot } if current_slot > slot + SWITCHBOARD_FUNCTION_SLOT_UNTIL_EXPIRATION as u64)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum SwitchboardFunctionRequestStatus {
    // No request has been made yet
    None,
    // The request has been made but the callback has not been called by the Function yet
    Requested { slot: u64 },
    // The request has been settled. the callback has been made
    Settled { slot: u64 },
    // The request has expired and was not settled
    Expired { slot: u64 },
}

// Ignore specific timestamp
impl PartialEq for SwitchboardFunctionRequestStatus {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (
                SwitchboardFunctionRequestStatus::None,
                SwitchboardFunctionRequestStatus::None
            ) | (
                SwitchboardFunctionRequestStatus::Requested { slot: _ },
                SwitchboardFunctionRequestStatus::Requested { slot: _ }
            ) | (
                SwitchboardFunctionRequestStatus::Settled { slot: _ },
                SwitchboardFunctionRequestStatus::Settled { slot: _ }
            )
        )
    }
}
