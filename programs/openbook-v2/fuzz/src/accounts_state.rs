use bumpalo::Bump;
use openbook_v2::state::*;
use solana_program::{
    account_info::AccountInfo, bpf_loader, clock::Epoch, instruction::AccountMeta,
    program_pack::Pack, pubkey::Pubkey, rent::Rent, system_program,
};
use solana_sdk::account::{Account, WritableAccount};
use spl_token::state::{Account as TokenAccount, Mint};
use std::collections::HashMap;

pub struct AccountsState(HashMap<Pubkey, Account>);

impl AccountsState {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn default() -> Self {
        Self::new()
    }

    pub fn insert(&mut self, pubkey: Pubkey, account: Account) {
        self.0.insert(pubkey, account);
    }

    pub fn account_infos<'a, 'b: 'a>(
        &'a self,
        bump: &'b Bump,
        metas: Vec<AccountMeta>,
    ) -> Vec<AccountInfo<'b>> {
        metas
            .iter()
            .map(|meta| {
                let account = self.0.get(&meta.pubkey).unwrap();
                AccountInfo::new(
                    bump.alloc(meta.pubkey),
                    meta.is_signer,
                    meta.is_writable,
                    bump.alloc(account.lamports),
                    bump.alloc_slice_copy(&account.data),
                    bump.alloc(account.owner),
                    account.executable,
                    account.rent_epoch,
                )
            })
            .collect()
    }

    pub fn update(&mut self, infos: Vec<AccountInfo>) {
        infos.iter().for_each(|info| {
            let account = self.0.get_mut(info.key).unwrap();
            let new_data = info.data.borrow();
            let new_lamports = **info.lamports.borrow();
            if new_lamports != account.lamports || *new_data != account.data {
                account.data = (*new_data).to_vec();
                account.lamports = new_lamports;
            }
        });
    }

    pub fn add_program(&mut self, pubkey: Pubkey) -> &mut Self {
        self.insert(
            pubkey,
            Account::create(0, vec![], bpf_loader::ID, true, Epoch::default()),
        );
        self
    }

    pub fn add_account_with_lamports(&mut self, pubkey: Pubkey, lamports: u64) -> &mut Self {
        self.insert(
            pubkey,
            Account::create(
                lamports,
                vec![],
                system_program::ID,
                false,
                Epoch::default(),
            ),
        );
        self
    }

    pub fn add_token_account(&mut self, pubkey: Pubkey, owner: Pubkey, mint: Pubkey) -> &mut Self {
        let mut data = vec![0_u8; TokenAccount::LEN];
        let mut account = TokenAccount::default();
        account.state = spl_token::state::AccountState::Initialized;
        account.mint = mint;
        account.owner = owner;
        TokenAccount::pack(account, &mut data).unwrap();
        self.insert(
            pubkey,
            Account::create(
                Rent::default().minimum_balance(data.len()),
                data,
                spl_token::ID,
                false,
                Epoch::default(),
            ),
        );
        self
    }

    pub fn add_mint(&mut self, pubkey: Pubkey) -> &mut Self {
        let mut data = vec![0_u8; Mint::LEN];
        let mut mint = Mint::default();
        mint.is_initialized = true;
        Mint::pack(mint, &mut data).unwrap();
        self.insert(
            pubkey,
            Account::create(
                Rent::default().minimum_balance(data.len()),
                data,
                spl_token::ID,
                false,
                Epoch::default(),
            ),
        );
        self
    }

    pub fn add_openbook_account<T>(&mut self, pubkey: Pubkey) -> &mut Self {
        let len = 8 + std::mem::size_of::<T>();
        self.insert(
            pubkey,
            Account::create(
                Rent::default().minimum_balance(len),
                vec![0; len],
                openbook_v2::ID,
                false,
                Epoch::default(),
            ),
        );
        self
    }
}
