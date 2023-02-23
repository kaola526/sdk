// Copyright (C) 2019-2023 Aleo Systems Inc.
// This file is part of the Aleo library.

// The Aleo library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The Aleo library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the Aleo library. If not, see <https://www.gnu.org/licenses/>.

use anyhow::Result;
use snarkvm_console::{
    account::PrivateKey,
    program::{Network, Plaintext, Record},
};
use snarkvm_synthesizer::{ConsensusMemory, ConsensusStore, Program, Query, Transaction, VM};

use super::ProgramManager;

impl<N: Network> ProgramManager<N> {
    pub fn create_deploy_transaction(
        private_key: PrivateKey<N>,
        fee: u64,
        record: Record<N, Plaintext<N>>,
        program: &Program<N>,
        query: String,
    ) -> Result<Transaction<N>> {
        // Initialize an RNG.
        let rng = &mut rand::thread_rng();

        // Initialize the VM.
        let store = ConsensusStore::<N, ConsensusMemory<N>>::open(None)?;
        let vm = VM::<N, ConsensusMemory<N>>::from(store)?;
        let query = Query::from(query);

        // Create the transaction
        Transaction::<N>::deploy(&vm, &private_key, program, (record, fee), Some(query), rng)
    }
}