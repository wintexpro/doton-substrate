#![cfg(test)]

use super::mock::{
    assert_events, new_test_ext, Balances, Bridge, Call, Event, Origin, ProposalLifetime, Dorr, run_to_block,
    Test, TestChainId, ENDOWED_BALANCE, RELAYER_A, RELAYER_B, RELAYER_C, TEST_THRESHOLD,
};
use super::*;
use crate::mock::new_test_ext_initialized;
use frame_support::{assert_noop, assert_ok};
use rustc_hex::{FromHex};

#[test]
fn derive_ids() {
    let chain = 1;
    let id = [
        0x21, 0x60, 0x5f, 0x71, 0x84, 0x5f, 0x37, 0x2a, 0x9e, 0xd8, 0x42, 0x53, 0xd2, 0xd0, 0x24,
        0xb7, 0xb1, 0x09, 0x99, 0xf4,
    ];
    let r_id = derive_resource_id(chain, &id);
    let expected = [
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x21, 0x60, 0x5f, 0x71, 0x84, 0x5f,
        0x37, 0x2a, 0x9e, 0xd8, 0x42, 0x53, 0xd2, 0xd0, 0x24, 0xb7, 0xb1, 0x09, 0x99, 0xf4, chain,
    ];
    assert_eq!(r_id, expected);
}

#[test]
fn complete_proposal_approved() {
    let mut prop = ProposalVotes {
        votes_for: vec![1, 2],
        votes_against: vec![3],
        status: ProposalStatus::Initiated,
        expiry: ProposalLifetime::get(),
    };

    prop.try_to_complete(2, 3);
    assert_eq!(prop.status, ProposalStatus::Approved);
}

#[test]
fn complete_proposal_rejected() {
    let mut prop = ProposalVotes {
        votes_for: vec![1],
        votes_against: vec![2, 3],
        status: ProposalStatus::Initiated,
        expiry: ProposalLifetime::get(),
    };

    prop.try_to_complete(2, 3);
    assert_eq!(prop.status, ProposalStatus::Rejected);
}

#[test]
fn complete_proposal_bad_threshold() {
    let mut prop = ProposalVotes {
        votes_for: vec![1, 2],
        votes_against: vec![],
        status: ProposalStatus::Initiated,
        expiry: ProposalLifetime::get(),
    };

    prop.try_to_complete(3, 2);
    assert_eq!(prop.status, ProposalStatus::Initiated);

    let mut prop = ProposalVotes {
        votes_for: vec![],
        votes_against: vec![1, 2],
        status: ProposalStatus::Initiated,
        expiry: ProposalLifetime::get(),
    };

    prop.try_to_complete(3, 2);
    assert_eq!(prop.status, ProposalStatus::Initiated);
}

#[test]
fn setup_resources() {
    new_test_ext().execute_with(|| {
        let id: ResourceId = [1; 32];
        let method = "Pallet.do_something".as_bytes().to_vec();
        let method2 = "Pallet.do_somethingElse".as_bytes().to_vec();

        assert_ok!(Bridge::set_resource(Origin::root(), id, method.clone()));
        assert_eq!(Bridge::resources(id), Some(method));

        assert_ok!(Bridge::set_resource(Origin::root(), id, method2.clone()));
        assert_eq!(Bridge::resources(id), Some(method2));

        assert_ok!(Bridge::remove_resource(Origin::root(), id));
        assert_eq!(Bridge::resources(id), None);
    })
}

#[test]
fn whitelist_chain() {
    new_test_ext().execute_with(|| {
        assert!(!Bridge::chain_whitelisted(0));

        assert_ok!(Bridge::whitelist_chain(Origin::root(), 0));
        assert_noop!(
            Bridge::whitelist_chain(Origin::root(), TestChainId::get()),
            Error::<Test>::InvalidChainId
        );

        assert_events(vec![Event::bridge(RawEvent::ChainWhitelisted(0))]);
    })
}

#[test]
fn set_get_threshold() {
    new_test_ext().execute_with(|| {
        assert_eq!(<RelayerThreshold>::get(), 1);

        assert_ok!(Bridge::set_threshold(Origin::root(), TEST_THRESHOLD));
        assert_eq!(<RelayerThreshold>::get(), TEST_THRESHOLD);

        assert_ok!(Bridge::set_threshold(Origin::root(), 5));
        assert_eq!(<RelayerThreshold>::get(), 5);

        assert_events(vec![
            Event::bridge(RawEvent::RelayerThresholdChanged(TEST_THRESHOLD)),
            Event::bridge(RawEvent::RelayerThresholdChanged(5)),
        ]);
    })
}

#[test]
fn asset_transfer_success() {
    new_test_ext().execute_with(|| {
        let dest_id = 2;
        let to = vec![2];
        let resource_id = [1; 32];
        let metadata = vec![];
        let amount = 100;
        let token_id = vec![1, 2, 3, 4];

        assert_ok!(Bridge::set_threshold(Origin::root(), TEST_THRESHOLD,));

        assert_ok!(Bridge::whitelist_chain(Origin::root(), dest_id.clone()));
        assert_ok!(Bridge::transfer_fungible(
            dest_id.clone(),
            resource_id.clone(),
            to.clone(),
            amount.into()
        ));
        assert_events(vec![
            Event::bridge(RawEvent::ChainWhitelisted(dest_id.clone())),
            Event::bridge(RawEvent::FungibleTransfer(
                dest_id.clone(),
                1,
                resource_id.clone(),
                amount.into(),
                to.clone(),
            )),
        ]);

        assert_ok!(Bridge::transfer_nonfungible(
            dest_id.clone(),
            resource_id.clone(),
            token_id.clone(),
            to.clone(),
            metadata.clone()
        ));
        assert_events(vec![Event::bridge(RawEvent::NonFungibleTransfer(
            dest_id.clone(),
            2,
            resource_id.clone(),
            token_id,
            to.clone(),
            metadata.clone(),
        ))]);

        assert_ok!(Bridge::transfer_generic(
            dest_id.clone(),
            resource_id.clone(),
            metadata.clone()
        ));
        assert_events(vec![Event::bridge(RawEvent::GenericTransfer(
            dest_id.clone(),
            3,
            resource_id,
            metadata,
        ))]);
    })
}

#[test]
fn asset_transfer_invalid_chain() {
    new_test_ext().execute_with(|| {
        let chain_id = 2;
        let bad_dest_id = 3;
        let resource_id = [4; 32];

        assert_ok!(Bridge::whitelist_chain(Origin::root(), chain_id.clone()));
        assert_events(vec![Event::bridge(RawEvent::ChainWhitelisted(
            chain_id.clone(),
        ))]);

        assert_noop!(
            Bridge::transfer_fungible(bad_dest_id, resource_id.clone(), vec![], U256::zero()),
            Error::<Test>::ChainNotWhitelisted
        );

        assert_noop!(
            Bridge::transfer_nonfungible(bad_dest_id, resource_id.clone(), vec![], vec![], vec![]),
            Error::<Test>::ChainNotWhitelisted
        );

        assert_noop!(
            Bridge::transfer_generic(bad_dest_id, resource_id.clone(), vec![]),
            Error::<Test>::ChainNotWhitelisted
        );
    })
}

#[test]
fn add_remove_relayer() {
    new_test_ext().execute_with(|| {
        assert_ok!(Bridge::set_threshold(Origin::root(), TEST_THRESHOLD,));
        assert_eq!(Bridge::relayer_count(), 0);

        assert_ok!(Bridge::add_relayer(Origin::root(), RELAYER_A));
        assert_ok!(Bridge::add_relayer(Origin::root(), RELAYER_B));
        assert_ok!(Bridge::add_relayer(Origin::root(), RELAYER_C));
        assert_eq!(Bridge::relayer_count(), 3);

        // Already exists
        assert_noop!(
            Bridge::add_relayer(Origin::root(), RELAYER_A),
            Error::<Test>::RelayerAlreadyExists
        );

        // Confirm removal
        assert_ok!(Bridge::remove_relayer(Origin::root(), RELAYER_B));
        assert_eq!(Bridge::relayer_count(), 2);
        assert_noop!(
            Bridge::remove_relayer(Origin::root(), RELAYER_B),
            Error::<Test>::RelayerInvalid
        );
        assert_eq!(Bridge::relayer_count(), 2);

        assert_events(vec![
            Event::bridge(RawEvent::RelayerAdded(RELAYER_A)),
            Event::bridge(RawEvent::RelayerAdded(RELAYER_B)),
            Event::bridge(RawEvent::RelayerAdded(RELAYER_C)),
            Event::bridge(RawEvent::RelayerRemoved(RELAYER_B)),
        ]);
    })
}

fn make_proposal(r: Vec<u8>) -> mock::Call {
    Call::System(system::Call::remark(r))
}

#[test]
fn create_sucessful_proposal() {
    let src_id = 1;
    let r_id = derive_resource_id(src_id, b"remark");

    new_test_ext_initialized(src_id, r_id, b"System.remark".to_vec()).execute_with(|| {
        let prop_id = 1;
        let proposal = make_proposal(vec![10]);

        let pk: String = String::from("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d");
        assert_ok!(Dorr::set_pk(Origin::signed(RELAYER_A), pk.from_hex().unwrap()));

        run_to_block(11);

        assert_ok!(Dorr::set_vrf_results(
          Origin::signed(RELAYER_A),
          String::from("dcd0f3a7d0af4a1336b7cad05ffedd3486ca88c6e32eb096b301dced2ae43f5a").from_hex().unwrap(),
          String::from("fa44dbfe6f3d4b49b623777b28412fac2168a463360ef0b531fdb70a76643b07910a8a616f00861a6399d70477918d5cf04e18a7fe298779eae862003027f302").from_hex().unwrap())
        );

        // Create proposal (& vote)
        assert_ok!(Bridge::acknowledge_proposal(
            Origin::signed(RELAYER_A),
            prop_id,
            src_id,
            r_id,
            Box::new(proposal.clone())
        ));
        let prop = Bridge::votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
        let expected = ProposalVotes {
            votes_for: vec![RELAYER_A],
            votes_against: vec![],
            status: ProposalStatus::Initiated,
            expiry: ProposalLifetime::get() + 11,
        };
        assert_eq!(prop, expected);

        let pk: String = String::from("8a3cac9282ba021cc6090f6ddfc826383300facb2101d6c736d794a5b25aa060");
        assert_ok!(Dorr::set_pk(Origin::signed(RELAYER_B), pk.from_hex().unwrap()));

        run_to_block(21);

        assert_ok!(Dorr::set_vrf_results(
          Origin::signed(RELAYER_B),
          String::from("10a343aaa12503ee7e004a7c56eb6f3956cba77f38a62a6e9544daa7ab07a913").from_hex().unwrap(),
          String::from("c8c4383bcf63585f6ae816e05779b13dc130af4c13c3869e5c59b4dffd0283046b3e62e7a09ed59a15246eeb7326046fcd48203d6a723375f5e9084fdba2a807").from_hex().unwrap())
        );

        // Second relayer votes against
        assert_ok!(Bridge::reject_proposal(
            Origin::signed(RELAYER_B),
            prop_id,
            src_id,
            r_id,
            Box::new(proposal.clone())
        ));
        let prop = Bridge::votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
        let expected = ProposalVotes {
            votes_for: vec![RELAYER_A],
            votes_against: vec![RELAYER_B],
            status: ProposalStatus::Initiated,
            expiry: ProposalLifetime::get() + 11,
        };
        assert_eq!(prop, expected);

        let pk: String = String::from("e062f3b7ff6d5de1339ccb295d7202c440e9d8bf421ca30ea8c37eae0a7ef559");
        assert_ok!(Dorr::set_pk(Origin::signed(RELAYER_C), pk.from_hex().unwrap()));

        run_to_block(31);

        assert_ok!(Dorr::set_vrf_results(
          Origin::signed(RELAYER_C),
          String::from("22ed779046c565be3c8f50c6e63cf7e6786a15de7283b86cac53528a2006516c").from_hex().unwrap(),
          String::from("f0e53ab529aa808395b50842237b7e6ebae0a0694046470aa66336cd4b8b6700846706d0bf9bfbfb72e82f54423c94b0871f9272d1763169921d9c6faa1a350d").from_hex().unwrap())
        );

        // Third relayer votes in favour
        assert_ok!(Bridge::acknowledge_proposal(
            Origin::signed(RELAYER_C),
            prop_id,
            src_id,
            r_id,
            Box::new(proposal.clone())
        ));
        let prop = Bridge::votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
        let expected = ProposalVotes {
            votes_for: vec![RELAYER_A, RELAYER_C],
            votes_against: vec![RELAYER_B],
            status: ProposalStatus::Approved,
            expiry: ProposalLifetime::get() + 11,
        };
        assert_eq!(prop, expected);

        assert_events(vec![
            Event::bridge(RawEvent::VoteFor(src_id, prop_id, RELAYER_A)),
            Event::bridge(RawEvent::VoteAgainst(src_id, prop_id, RELAYER_B)),
            Event::bridge(RawEvent::VoteFor(src_id, prop_id, RELAYER_C)),
            Event::bridge(RawEvent::ProposalApproved(src_id, prop_id)),
            Event::bridge(RawEvent::ProposalSucceeded(src_id, prop_id)),
        ]);
    })
}

#[test]
fn create_unsucessful_proposal() {
    let src_id = 1;
    let r_id = derive_resource_id(src_id, b"transfer");

    new_test_ext_initialized(src_id, r_id, b"System.remark".to_vec()).execute_with(|| {
        let prop_id = 1;
        let proposal = make_proposal(vec![11]);

        let pk: String = String::from("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d");
        assert_ok!(Dorr::set_pk(Origin::signed(RELAYER_A), pk.from_hex().unwrap()));

        run_to_block(11);

        assert_ok!(Dorr::set_vrf_results(
          Origin::signed(RELAYER_A),
          String::from("dcd0f3a7d0af4a1336b7cad05ffedd3486ca88c6e32eb096b301dced2ae43f5a").from_hex().unwrap(),
          String::from("fa44dbfe6f3d4b49b623777b28412fac2168a463360ef0b531fdb70a76643b07910a8a616f00861a6399d70477918d5cf04e18a7fe298779eae862003027f302").from_hex().unwrap())
        );

        // Create proposal (& vote)
        assert_ok!(Bridge::acknowledge_proposal(
            Origin::signed(RELAYER_A),
            prop_id,
            src_id,
            r_id,
            Box::new(proposal.clone())
        ));
        let prop = Bridge::votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
        let expected = ProposalVotes {
            votes_for: vec![RELAYER_A],
            votes_against: vec![],
            status: ProposalStatus::Initiated,
            expiry: ProposalLifetime::get() + 11,
        };
        assert_eq!(prop, expected);

        let pk: String = String::from("8a3cac9282ba021cc6090f6ddfc826383300facb2101d6c736d794a5b25aa060");
        assert_ok!(Dorr::set_pk(Origin::signed(RELAYER_B), pk.from_hex().unwrap()));

        run_to_block(21);

        assert_ok!(Dorr::set_vrf_results(
          Origin::signed(RELAYER_B),
          String::from("10a343aaa12503ee7e004a7c56eb6f3956cba77f38a62a6e9544daa7ab07a913").from_hex().unwrap(),
          String::from("c8c4383bcf63585f6ae816e05779b13dc130af4c13c3869e5c59b4dffd0283046b3e62e7a09ed59a15246eeb7326046fcd48203d6a723375f5e9084fdba2a807").from_hex().unwrap())
        );

        // Second relayer votes against
        assert_ok!(Bridge::reject_proposal(
            Origin::signed(RELAYER_B),
            prop_id,
            src_id,
            r_id,
            Box::new(proposal.clone())
        ));
        let prop = Bridge::votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
        let expected = ProposalVotes {
            votes_for: vec![RELAYER_A],
            votes_against: vec![RELAYER_B],
            status: ProposalStatus::Initiated,
            expiry: ProposalLifetime::get() + 11,
        };
        assert_eq!(prop, expected);

        let pk: String = String::from("e062f3b7ff6d5de1339ccb295d7202c440e9d8bf421ca30ea8c37eae0a7ef559");
        assert_ok!(Dorr::set_pk(Origin::signed(RELAYER_C), pk.from_hex().unwrap()));

        run_to_block(31);

        assert_ok!(Dorr::set_vrf_results(
          Origin::signed(RELAYER_C),
          String::from("22ed779046c565be3c8f50c6e63cf7e6786a15de7283b86cac53528a2006516c").from_hex().unwrap(),
          String::from("f0e53ab529aa808395b50842237b7e6ebae0a0694046470aa66336cd4b8b6700846706d0bf9bfbfb72e82f54423c94b0871f9272d1763169921d9c6faa1a350d").from_hex().unwrap())
        );

        // Third relayer votes against
        assert_ok!(Bridge::reject_proposal(
            Origin::signed(RELAYER_C),
            prop_id,
            src_id,
            r_id,
            Box::new(proposal.clone())
        ));
        let prop = Bridge::votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
        let expected = ProposalVotes {
            votes_for: vec![RELAYER_A],
            votes_against: vec![RELAYER_B, RELAYER_C],
            status: ProposalStatus::Rejected,
            expiry: ProposalLifetime::get() + 11,
        };
        assert_eq!(prop, expected);

        assert_eq!(Balances::free_balance(RELAYER_B), 0);
        assert_eq!(
            Balances::free_balance(Bridge::account_id()),
            ENDOWED_BALANCE
        );

        assert_events(vec![
            Event::bridge(RawEvent::VoteFor(src_id, prop_id, RELAYER_A)),
            Event::bridge(RawEvent::VoteAgainst(src_id, prop_id, RELAYER_B)),
            Event::bridge(RawEvent::VoteAgainst(src_id, prop_id, RELAYER_C)),
            Event::bridge(RawEvent::ProposalRejected(src_id, prop_id)),
        ]);
    })
}

#[test]
fn execute_after_threshold_change() {
    let src_id = 1;
    let r_id = derive_resource_id(src_id, b"transfer");

    new_test_ext_initialized(src_id, r_id, b"System.remark".to_vec()).execute_with(|| {
        let prop_id = 1;
        let proposal = make_proposal(vec![11]);

        let pk: String = String::from("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d");
        assert_ok!(Dorr::set_pk(Origin::signed(RELAYER_A), pk.from_hex().unwrap()));

        run_to_block(11);

        assert_ok!(Dorr::set_vrf_results(
          Origin::signed(RELAYER_A),
          String::from("dcd0f3a7d0af4a1336b7cad05ffedd3486ca88c6e32eb096b301dced2ae43f5a").from_hex().unwrap(),
          String::from("fa44dbfe6f3d4b49b623777b28412fac2168a463360ef0b531fdb70a76643b07910a8a616f00861a6399d70477918d5cf04e18a7fe298779eae862003027f302").from_hex().unwrap())
        );

        // Create proposal (& vote)
        assert_ok!(Bridge::acknowledge_proposal(
            Origin::signed(RELAYER_A),
            prop_id,
            src_id,
            r_id,
            Box::new(proposal.clone())
        ));
        let prop = Bridge::votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
        let expected = ProposalVotes {
            votes_for: vec![RELAYER_A],
            votes_against: vec![],
            status: ProposalStatus::Initiated,
            expiry: ProposalLifetime::get() + 11,
        };
        assert_eq!(prop, expected);

        // Change threshold
        assert_ok!(Bridge::set_threshold(Origin::root(), 1));

        // Attempt to execute
        assert_ok!(Bridge::eval_vote_state(
            Origin::signed(RELAYER_A),
            prop_id,
            src_id,
            Box::new(proposal.clone())
        ));

        let prop = Bridge::votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
        let expected = ProposalVotes {
            votes_for: vec![RELAYER_A],
            votes_against: vec![],
            status: ProposalStatus::Approved,
            expiry: ProposalLifetime::get() + 11,
        };
        assert_eq!(prop, expected);

        assert_eq!(Balances::free_balance(RELAYER_B), 0);
        assert_eq!(
            Balances::free_balance(Bridge::account_id()),
            ENDOWED_BALANCE
        );

        assert_events(vec![
            Event::bridge(RawEvent::VoteFor(src_id, prop_id, RELAYER_A)),
            Event::bridge(RawEvent::RelayerThresholdChanged(1)),
            Event::bridge(RawEvent::ProposalApproved(src_id, prop_id)),
            Event::bridge(RawEvent::ProposalSucceeded(src_id, prop_id)),
        ]);
    })
}

#[test]
fn proposal_expires() {
    let src_id = 1;
    let r_id = derive_resource_id(src_id, b"remark");

    new_test_ext_initialized(src_id, r_id, b"System.remark".to_vec()).execute_with(|| {
        let prop_id = 1;
        let proposal = make_proposal(vec![10]);

        let pk: String = String::from("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d");
        assert_ok!(Dorr::set_pk(Origin::signed(RELAYER_A), pk.from_hex().unwrap()));

        run_to_block(11);

        assert_ok!(Dorr::set_vrf_results(
          Origin::signed(RELAYER_A),
          String::from("dcd0f3a7d0af4a1336b7cad05ffedd3486ca88c6e32eb096b301dced2ae43f5a").from_hex().unwrap(),
          String::from("fa44dbfe6f3d4b49b623777b28412fac2168a463360ef0b531fdb70a76643b07910a8a616f00861a6399d70477918d5cf04e18a7fe298779eae862003027f302").from_hex().unwrap())
        );

        // Create proposal (& vote)
        assert_ok!(Bridge::acknowledge_proposal(
            Origin::signed(RELAYER_A),
            prop_id,
            src_id,
            r_id,
            Box::new(proposal.clone())
        ));
        let prop = Bridge::votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
        let expected = ProposalVotes {
            votes_for: vec![RELAYER_A],
            votes_against: vec![],
            status: ProposalStatus::Initiated,
            expiry: ProposalLifetime::get() + 11,
        };
        assert_eq!(prop, expected);

        let pk: String = String::from("8a3cac9282ba021cc6090f6ddfc826383300facb2101d6c736d794a5b25aa060");
        assert_ok!(Dorr::set_pk(Origin::signed(RELAYER_B), pk.from_hex().unwrap()));

        // Increment enough blocks such that now == expiry
        run_to_block(61);

        assert_ok!(Dorr::set_vrf_results(
          Origin::signed(RELAYER_B),
          String::from("10a343aaa12503ee7e004a7c56eb6f3956cba77f38a62a6e9544daa7ab07a913").from_hex().unwrap(),
          String::from("c8c4383bcf63585f6ae816e05779b13dc130af4c13c3869e5c59b4dffd0283046b3e62e7a09ed59a15246eeb7326046fcd48203d6a723375f5e9084fdba2a807").from_hex().unwrap())
        );

        // Attempt to submit a vote should fail
        assert_noop!(
            Bridge::reject_proposal(
                Origin::signed(RELAYER_B),
                prop_id,
                src_id,
                r_id,
                Box::new(proposal.clone())
            ),
            Error::<Test>::ProposalExpired
        );

        // Proposal state should remain unchanged
        let prop = Bridge::votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
        let expected = ProposalVotes {
            votes_for: vec![RELAYER_A],
            votes_against: vec![],
            status: ProposalStatus::Initiated,
            expiry: ProposalLifetime::get() + 11,
        };
        assert_eq!(prop, expected);

        // eval_vote_state should have no effect
        assert_noop!(
            Bridge::eval_vote_state(
                Origin::signed(RELAYER_C),
                prop_id,
                src_id,
                Box::new(proposal.clone())
            ),
            Error::<Test>::ProposalExpired
        );
        let prop = Bridge::votes(src_id, (prop_id.clone(), proposal.clone())).unwrap();
        let expected = ProposalVotes {
            votes_for: vec![RELAYER_A],
            votes_against: vec![],
            status: ProposalStatus::Initiated,
            expiry: ProposalLifetime::get() + 11,
        };
        assert_eq!(prop, expected);

        assert_events(vec![Event::bridge(RawEvent::VoteFor(
            src_id, prop_id, RELAYER_A,
        ))]);
    })
}
