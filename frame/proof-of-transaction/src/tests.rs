
use super::*;
use crate as pallet_pot;
use frame_support::{
    assert_noop, assert_ok,
    weights::Weight,
};
use mock::*;

struct ExtBuilder {
    balance_factor: u64,
}

impl ExtBuilder {
    
    pub fn default() -> Self {
        Self {
            balance_factor: 1,
        }
    }

    pub fn build(&self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<TestRuntime>()
            .unwrap();
        pallet_balances::GenesisConfig::<TestRuntime> {
            balances: if self.balance_factor > 0 {
                vec![
					(1, 10 * self.balance_factor),
					(2, 20 * self.balance_factor),
				]
            } else {
                vec![]
            },
        }
        .assimilate_storage(&mut t)
        .unwrap();

        t.into()
    }
}

/// create a transaction info struct from weight. Handy to avoid building the whole struct.
pub fn info_from_weight(w: Weight) -> DispatchInfo {
	// pays_fee: Pays::Yes -- class: DispatchClass::Normal
	DispatchInfo { weight: w, ..Default::default() }
}

fn default_post_info() -> PostDispatchInfo {
	PostDispatchInfo { actual_weight: None, pays_fee: Default::default() }
}

#[test]
fn signed_extension_works() {
    ExtBuilder::default()
        .build()
        .execute_with(|| {
            let caller = 1;
            let whom_to_vote = 1;
            let len = 10;
            let info = info_from_weight(Weight::from_parts(5, 0));
            let pre = CheckVote::<TestRuntime>::from(Some(whom_to_vote))
                .pre_dispatch(&caller, &CALL, &info, len)
                .unwrap();

            let post_info = &default_post_info();
            println!("{:?}", post_info);
            assert_ok!(
                CheckVote::<TestRuntime>::post_dispatch(
                    Some(pre), 
                    &info, 
                    post_info, 
                    len, 
                    &Ok(())
                )
            );
            assert!(
                VoteInfo::<TestRuntime>::contains_key(&1)
            );
            let weight = VoteInfo::<TestRuntime>::get(&1).unwrap();
            assert_eq!(
                weight,
                5
            )
        })
}