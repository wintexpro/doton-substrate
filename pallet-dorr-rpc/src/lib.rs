use std::sync::Arc;

use jsonrpc_derive::rpc;
use jsonrpc_core::{ Error as RpcError, ErrorCode, Result };

use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{ generic::BlockId, traits::{ Block as BlockT } };
use sp_core;

use dorr_pallet::{ DorrRuntimeApi };

/// An index to a block.
pub type BlockNumber = u32;

pub type Hash = sp_core::H256;

#[rpc]
pub trait DorrRpcApi<BlockHash, BlockNumber, Hash> {
	#[rpc(name = "dorr_getActiveRelayers")]
	fn get_active_relayers(
			&self,
			at: Option<BlockHash>
	) -> Result<Vec<Vec<u8>>>;

	#[rpc(name = "dorr_isActivePk")]
	fn is_active_pk(
			&self,
			pk: Vec<u8>,
			at: Option<BlockHash>,
	) -> Result<bool>;

	#[rpc(name = "dorr_getEpochByPk")]
	fn get_epoch_by_pk(
			&self,
			pk: Vec<u8>,
			at: Option<BlockHash>,
	) -> Result<BlockNumber>;

	#[rpc(name = "dorr_getCurrentEpoch")]
	fn get_current_epoch(
			&self,
			at: Option<BlockHash>,
	) -> Result<BlockNumber>;

	#[rpc(name = "dorr_getPublicRandomness")]
	fn get_public_randomness(
			&self,
			epoch: BlockNumber,
			at: Option<BlockHash>,
	) -> Result<Hash>;
}

/// A struct that implements the `DorrApi`.
pub struct DorrRpc<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> DorrRpc<C, M> {
	/// Create new `DorrRpc` instance with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
			Self { client, _marker: Default::default() }
	}
}

impl<C, Block> DorrRpcApi<<Block as BlockT>::Hash, BlockNumber, Hash>
	for DorrRpc<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: DorrRuntimeApi<Block>,
{
	fn get_active_relayers(
		&self,
		at: Option<<Block as BlockT>::Hash>
	) -> Result<Vec<Vec<u8>>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash
		));

			let runtime_api_result = api.get_active_relayers(&at);

			runtime_api_result.map_err(|e| RpcError {
				code: ErrorCode::ServerError(100500),
				message: "Something wrong".into(),
				data: Some(format!("{:?}", e).into()),
			})
	}

	fn is_active_pk(
		&self,
		pk: Vec<u8>,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<bool> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash
		));

			let runtime_api_result = api.is_active_pk(&at, pk);

			runtime_api_result.map_err(|e| RpcError {
				code: ErrorCode::ServerError(100501),
				message: "Something wrong".into(),
				data: Some(format!("{:?}", e).into()),
			})
	}

	fn get_epoch_by_pk(
		&self,
		pk: Vec<u8>,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<BlockNumber> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash
		));

			let runtime_api_result = api.get_epoch_by_pk(&at, pk);

			runtime_api_result.map_err(|e| RpcError {
				code: ErrorCode::ServerError(100502),
				message: "Something wrong".into(),
				data: Some(format!("{:?}", e).into()),
			})
	}

	fn get_current_epoch(
		&self,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<BlockNumber> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash
		));

			let runtime_api_result = api.get_current_epoch(&at);

			runtime_api_result.map_err(|e| RpcError {
				code: ErrorCode::ServerError(100502),
				message: "Something wrong".into(),
				data: Some(format!("{:?}", e).into()),
			})
	}

	fn get_public_randomness(
		&self,
		epoch: BlockNumber,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Hash> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash
		));

			let runtime_api_result = api.get_public_randomness(&at, epoch);

			runtime_api_result.map_err(|e| RpcError {
				code: ErrorCode::ServerError(100502),
				message: "Something wrong".into(),
				data: Some(format!("{:?}", e).into()),
			})
	}
}
