#![cfg(test)]

extern crate rustc_hex;
use frame_support::{assert_ok};
use super::mock::{ALICE, BOB, CHARLIE, Origin, Dorr, new_test_ext, run_to_block};
use rustc_hex::{FromHex};

#[test]
fn current_epoch_calculating_should_work() {
	for n in 1..121 {
		new_test_ext(n).execute_with(|| {
			let e = (n + 5 - 1) / 5;
			assert_eq!(Dorr::get_current_epoch(), e);
		});
	}
}

#[test]
fn set_vrf_results_should_work() {
	new_test_ext(1).execute_with(|| {
		let pk: String = String::from("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d");
		assert_ok!(Dorr::set_pk(Origin::signed(ALICE), pk.from_hex().unwrap()));

		let pk: String = String::from("8a3cac9282ba021cc6090f6ddfc826383300facb2101d6c736d794a5b25aa060");
		assert_ok!(Dorr::set_pk(Origin::signed(BOB), pk.from_hex().unwrap()));

		let pk: String = String::from("e062f3b7ff6d5de1339ccb295d7202c440e9d8bf421ca30ea8c37eae0a7ef559");
		assert_ok!(Dorr::set_pk(Origin::signed(CHARLIE), pk.from_hex().unwrap()));

		run_to_block(6);

		assert_ok!(Dorr::set_vrf_results(
			Origin::signed(ALICE),
			String::from("dcd0f3a7d0af4a1336b7cad05ffedd3486ca88c6e32eb096b301dced2ae43f5a").from_hex().unwrap(),
			String::from("fa44dbfe6f3d4b49b623777b28412fac2168a463360ef0b531fdb70a76643b07910a8a616f00861a6399d70477918d5cf04e18a7fe298779eae862003027f302").from_hex().unwrap())
		);

		run_to_block(11);

		assert_ok!(Dorr::set_vrf_results(
			Origin::signed(BOB),
			String::from("10a343aaa12503ee7e004a7c56eb6f3956cba77f38a62a6e9544daa7ab07a913").from_hex().unwrap(),
			String::from("c8c4383bcf63585f6ae816e05779b13dc130af4c13c3869e5c59b4dffd0283046b3e62e7a09ed59a15246eeb7326046fcd48203d6a723375f5e9084fdba2a807").from_hex().unwrap())
		);

		run_to_block(16);

		assert_ok!(Dorr::set_vrf_results(
			Origin::signed(CHARLIE),
			String::from("22ed779046c565be3c8f50c6e63cf7e6786a15de7283b86cac53528a2006516c").from_hex().unwrap(),
			String::from("f0e53ab529aa808395b50842237b7e6ebae0a0694046470aa66336cd4b8b6700846706d0bf9bfbfb72e82f54423c94b0871f9272d1763169921d9c6faa1a350d").from_hex().unwrap())
		);
	});
}