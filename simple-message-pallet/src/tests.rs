#![cfg(test)]

use super::*;
use frame_support::{assert_ok};
use super::mock::{new_test_ext, Origin, Call, Bridge, RELAYER_A};

#[test]
fn writeing_msg_should_work() {
  new_test_ext().execute_with(|| {

    let msg: Vec<u8> = vec![104, 101, 108, 108, 111];

    let proposal = Call::SimpleMsg(crate::Call::write_msg(0, msg));
    let prop_id = 1;
    let src_id = 1;
    let r_id = bridge::derive_resource_id(src_id, b"hash");
    let resource = b"SimpleMsg.write_msg".to_vec();

    assert_ok!(Bridge::add_relayer(Origin::root(), RELAYER_A));
    assert_ok!(Bridge::whitelist_chain(Origin::root(), src_id));
    assert_ok!(Bridge::set_resource(Origin::root(), r_id, resource));

    assert_ok!(Bridge::acknowledge_proposal(
      Origin::signed(RELAYER_A),
      prop_id,
      src_id,
      r_id,
      Box::new(proposal.clone())
  ));

    // assert_ok!();

    // let (_, _ , recived_msg_vec) = <Messages<Test>>::get(0);
    // let recived_msg = String::from_utf8(recived_msg_vec).unwrap();

    // assert_eq!(recived_msg, "hello");
  });
}
