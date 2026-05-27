use anchor_lang::{
    prelude::*,
    solana_program::hash::hashv,
};

pub fn derive_random_u64(
    slot_hashes: &UncheckedAccount,
    unix_timestamp: i64,
    user: &Pubkey,
) -> u64 {
    let data = slot_hashes.data.borrow();

    let recent_hash = &data[16..48];

    let hash = hashv(&[
        recent_hash,
        &unix_timestamp.to_le_bytes(),
        user.as_ref(),
    ]);

    u64::from_le_bytes(
        hash.to_bytes()[..8]
            .try_into()
            .unwrap(),
    )
}
