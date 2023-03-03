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

use crate::{RecordQuery, Resolver};
use snarkvm::{
    file::{AleoFile, Manifest},
    package::Package,
};
use snarkvm_console::{
    account::PrivateKey,
    network::Network,
    program::{Owner::Public, Plaintext, ProgramID, Record},
};
use snarkvm_synthesizer::Program;

use anyhow::{ensure, Result};
use snarkvm_console::program::Address;
use std::{
    fs,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
};

/// Resolver for imports from the local file system
#[derive(Clone, Debug)]
pub struct FileSystemResolver<N: Network> {
    local_config: PathBuf,
    _phantom: core::marker::PhantomData<N>,
}

impl<N: Network> FileSystemResolver<N> {
    /// Create a new file system resolver
    pub fn new(local_config: &Path) -> Result<Self> {
        ensure!(local_config.exists(), "Path does not exist");
        ensure!(local_config.is_dir(), "Path is not a directory");
        Ok(Self { local_config: local_config.to_path_buf(), _phantom: core::marker::PhantomData })
    }

    pub fn import_directory(&self) -> PathBuf {
        self.local_config.join("imports")
    }

    pub fn inputs_directory(&self) -> PathBuf {
        self.local_config.join("inputs")
    }
}

impl<N: Network> Resolver<N> for FileSystemResolver<N> {
    const NAME: &'static str = "FileSystemResolver";

    fn load_program(&self, program_id: &ProgramID<N>) -> Result<Program<N>> {
        // Ensure the directory path exists.
        ensure!(self.local_config.exists(), "The program directory does not exist");

        ensure!(!Program::is_reserved_keyword(program_id.name()), "Program name is invalid (reserved): {program_id}");

        ensure!(
            Manifest::<N>::exists_at(&self.local_config),
            "Please ensure that the manifest file exists in the Aleo program directory (missing '{}' at '{}')",
            Manifest::<N>::file_name(),
            &self.local_config.display()
        );

        // Open the manifest file.
        let manifest = Manifest::<N>::open(&self.local_config)?;

        // Ensure the program ID matches the manifest program ID, or that the program is a local import
        if manifest.program_id() == program_id {
            // Load the package.
            let package = Package::open(&self.local_config)?;
            // Load the main program.
            Ok(package.program().clone())
        } else {
            let import_file = self.import_directory().join(program_id.to_string());
            ensure!(
                import_file.exists(),
                "No program named {:?} found at {:?}",
                program_id,
                self.local_config.display()
            );
            println!("Attempting to load program {:?} at {:?}", program_id, import_file.display());
            let mut program_file = File::open(import_file)?;
            let mut program_string = String::new();
            program_file.read_to_string(&mut program_string).map_err(|err| anyhow::anyhow!(err.to_string()))?;
            let program = Program::from_str(&program_string)?;
            println!("Loaded program {:?} successfully!", program_id);
            Ok(program)
        }
    }

    fn resolve_program_imports(&self, program: &Program<N>) -> Result<Vec<(ProgramID<N>, Result<Program<N>>)>> {
        program
            .imports()
            .keys()
            .map(|program_id| {
                // Open the Aleo program file.
                let program = AleoFile::open(&self.import_directory(), program_id, false)
                    .map(|aleo_file| aleo_file.program().clone())
                    .map_err(|err| anyhow::anyhow!(err.to_string()));
                Ok((*program_id, program))
            })
            .collect::<Result<Vec<(ProgramID<N>, Result<Program<N>>)>>>()
    }

    fn find_owned_records(
        &self,
        private_key: &PrivateKey<N>,
        _record_query: &RecordQuery,
    ) -> Result<Vec<Record<N, Plaintext<N>>>> {
        let mut records = vec![];
        let address = Address::try_from(private_key)?;
        for entry in fs::read_dir(&self.inputs_directory())? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "json" {
                        fs::read_to_string(path)
                            .map_err(|err| anyhow::anyhow!(err.to_string()))
                            .and_then(|json| {
                                serde_json::from_str::<Record<N, Plaintext<N>>>(&json)
                                    .map_err(|err| anyhow::anyhow!(err.to_string()))
                            })
                            .map(|record| {
                                let record_owner = record.owner();
                                if let Public(record_owner) = record_owner {
                                    if &address == record_owner {
                                        records.push(record.clone());
                                    }
                                }
                            })?;
                    } else {
                        continue;
                    }
                }
            }
        }
        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{random_string, setup_directory, teardown_directory, ALEO_PROGRAM, HELLO_PROGRAM};
    use snarkvm_console::network::Testnet3;
    use std::{ops::Add, panic::catch_unwind, str::FromStr};

    #[test]
    fn test_file_resolver_loading_and_imports() {
        let credits = Program::<Testnet3>::credits().unwrap().to_string();
        let imports = vec![("credits.aleo", credits.as_str()), ("hello.aleo", HELLO_PROGRAM)];
        let test_path = setup_directory("aleo_test_file_resolver", ALEO_PROGRAM, imports).unwrap();
        let result = catch_unwind(|| {
            // TEST 1: Test that the file resolver can load a program.
            println!("Test path: {}", test_path.display());
            let resolver = FileSystemResolver::<Testnet3>::new(&test_path).unwrap();
            let program_id = ProgramID::<Testnet3>::from_str("aleo_test.aleo").unwrap();
            let expected_program = Program::<Testnet3>::from_str(ALEO_PROGRAM).unwrap();
            let found_program = resolver.load_program(&program_id).unwrap();
            assert_eq!(expected_program, found_program);

            // TEST 2: Test that the file resolver can resolve imports.
            let test_program = Program::<Testnet3>::from_str(ALEO_PROGRAM).unwrap();
            let hello_program = Program::<Testnet3>::from_str(HELLO_PROGRAM).unwrap();
            let credits_program = Program::<Testnet3>::credits().unwrap();
            let imports = resolver.resolve_program_imports(&test_program).unwrap();
            assert_eq!(imports.len(), 2);

            let (hello_id, local_hello_program) = &imports[0];
            let (credits_id, local_credits_program) = &imports[1];
            let (local_hello_program, local_credits_program) =
                (local_hello_program.as_ref().unwrap(), local_credits_program.as_ref().unwrap());
            assert_eq!(hello_id.to_string(), "hello.aleo");
            assert_eq!(credits_id.to_string(), "credits.aleo");
            assert_eq!(&hello_program, local_hello_program);
            assert_eq!(&credits_program, local_credits_program);

            // TEST 3: Test that the file resolver doesn't load a non-existent program.
            let random_program = random_string(16);
            let program_id = ProgramID::<Testnet3>::from_str(&random_program).unwrap();
            assert!(resolver.load_program(&program_id).is_err());

            // TEST 4: Test that the file resolver throws an error when a program has a bad import, but can still resolve the other imports.
            // Create a bad program with a bad import
            let bad_import_code = String::from("import ").add(&random_string(16)).add(";").add(ALEO_PROGRAM);
            let bad_import_program = Program::<Testnet3>::from_str(&bad_import_code).unwrap();
            let imports = resolver.resolve_program_imports(&bad_import_program).unwrap();

            // Ensure that the bad import is the only one that failed
            let (_, local_bad_import_program) = &imports[0];
            let (hello_id, local_hello_program) = &imports[1];
            let (credits_id, local_credits_program) = &imports[2];
            assert!(local_bad_import_program.is_err());
            assert_eq!(hello_id.to_string(), "hello.aleo");
            assert_eq!(credits_id.to_string(), "credits.aleo");

            // Make sure the other imports are still resolved correctly
            let hello_program = Program::<Testnet3>::from_str(HELLO_PROGRAM).unwrap();
            let credits_program = Program::<Testnet3>::credits().unwrap();
            let (local_hello_program, local_credits_program) =
                (local_hello_program.as_ref().unwrap(), local_credits_program.as_ref().unwrap());

            assert_eq!(&hello_program, local_hello_program);
            assert_eq!(&credits_program, local_credits_program);

            // TEST 5: Ensure the file resolver doesn't resolve imports for a program that doesn't have any.
            let credits = Program::<Testnet3>::credits().unwrap();
            let imports = resolver.resolve_program_imports(&credits).unwrap();
            assert_eq!(imports.len(), 0);
        });
        teardown_directory(&test_path);
        // Ensure the test directory was deleted
        assert!(!test_path.exists());
        result.unwrap();
    }
}