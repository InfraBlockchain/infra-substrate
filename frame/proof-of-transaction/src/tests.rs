
use crate::{mock::*, VoteStatus, Vote};

#[test]
fn vote_collecting_works() {
    new_test_ext().execute_with(|| {
        // Origin: AccountId(1)
        // VoteTo: AccountId(2)
        assert_ok!(TestModule::test_vote(RuntimeOrigin::signed(1), 2));
        assert!(VoteStatus::<Test>::contains_key(&2));
        let vote_status = VoteStatus::<Test>::get(&2).unwrap();
        assert_eq!(
            vote_status,
            Vote {
                who: 2 as u64,
                count: 1 as u32
            }
        )
    })
}