use anchor_lang::prelude::*;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey, program_pack::IsInitialized,
};
use solana_program::program_pack::{Pack, Sealed};

entrypoint!(process_instruction);

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[derive(Debug, Default, PartialEq)]
pub struct Donation {
    donor_name: [u8; 32],
    blood_type: [u8; 3],
    date: u64,
}

impl Sealed for Donation {}

impl Pack for Donation {
    const LEN: usize = 40;

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let donor_name = src[..32].try_into().map_err(|_| ProgramError::InvalidInstructionData)?;
        let blood_type = src[32..35].try_into().map_err(|_| ProgramError::InvalidInstructionData)?;
        let date = u64::from_le_bytes(src[35..].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);

        Ok(Donation {
            donor_name,
            blood_type,
            date,
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        dst[..32].copy_from_slice(&self.donor_name);
        dst[32..35].copy_from_slice(&self.blood_type);
        dst[35..].copy_from_slice(&self.date.to_le_bytes());
    }
}


impl IsInitialized for Donation {
    fn is_initialized(&self) -> bool {
        // Implement initialization check logic
        // ...
        // Return true if the struct is properly initialized, false otherwise
        true
    }
}

#[program]
pub mod bloodchain_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if instruction_data.is_empty() {
        msg!("Instruction data is empty");
        return Err(ProgramError::InvalidInstructionData);
    }

    // Parse the instruction data and perform actions based on the instruction type
    match instruction_data[0] {
        0 => add_donation(accounts, &instruction_data[1..]),
        1 => retrieve_donation_history(accounts),
        _ => {
            msg!("Invalid instruction");
            Err(ProgramError::InvalidInstructionData)
        }
    }
}

fn add_donation(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    // Ensure the accounts are provided
    if accounts.is_empty() {
        msg!("No accounts provided");
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    // Retrieve the account info for the blood donation history state account
    let accounts_iter = &mut accounts.iter();
    let blood_donation_account = next_account_info(accounts_iter)?;

    // Deserialize the donation data
    let donation: Donation = unpack_donation(data)?;

    // Update the blood donation history with the new donation
    let mut blood_donation_history = get_donation_history(blood_donation_account)?;
    blood_donation_history.push(donation);

    // Serialize and save the updated blood donation history to the account data
    let blood_donation_history_data = pack_donation_history(&blood_donation_history)?;
    blood_donation_account.data.borrow_mut().copy_from_slice(&blood_donation_history_data);

    msg!("Donation added successfully");
    Ok(())
}

fn retrieve_donation_history(accounts: &[AccountInfo]) -> ProgramResult {
    // Ensure the accounts are provided
    if accounts.is_empty() {
        msg!("No accounts provided");
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    // Retrieve the account info for the blood donation history state account
    let accounts_iter = &mut accounts.iter();
    let blood_donation_account = next_account_info(accounts_iter)?;

    // Read the blood donation history from the account
    let blood_donation_history = get_donation_history(blood_donation_account)?;

    msg!("Blood Donation History:");
    for (index, donation) in blood_donation_history.iter().enumerate() {
        msg!(
            "Donation {}: Donor Name: {:?}, Blood Type: {:?}, Date: {}",
            index + 1,
            String::from_utf8_lossy(&donation.donor_name),
            String::from_utf8_lossy(&donation.blood_type),
            donation.date
        );
    }

    Ok(())
}

    // Helper function to deserialize donation data
fn unpack_donation(data: &[u8]) -> Result<Donation, ProgramError> {
    let donor_name: [u8; 32] = data[..32].try_into().map_err(|_| ProgramError::InvalidInstructionData)?;
    let blood_type: [u8; 3] = data[32..35].try_into().map_err(|_| ProgramError::InvalidInstructionData)?;
    let date = u64::from_le_bytes(data[35..].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);

    Ok(Donation {
        donor_name,
        blood_type,
        date,
    })
}

// Helper function to serialize donation history
fn pack_donation_history(history: &[Donation]) -> Result<Vec<u8>, ProgramError> {
    let mut result = Vec::new();
    for donation in history {
        result.extend_from_slice(&donation.donor_name);
        result.extend_from_slice(&donation.blood_type);
        result.extend_from_slice(&donation.date.to_le_bytes());
    }
    Ok(result)
}

// Helper function to get the current donation history from an account
fn get_donation_history(account: &AccountInfo) -> Result<Vec<Donation>, ProgramError> {
    let data = &account.data.borrow();
    let mut history = Vec::new();
    let mut index = 0;
    while index < data.len() {
        let donation_data = &data[index..(index + Donation::LEN)];
        let donation = Donation::unpack(donation_data)?;
        history.push(donation);
        index += Donation::LEN;
    }
    Ok(history)
}

#[cfg(test)]
mod tests {
    // Add unit tests
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 40)]
    blood_donation_account: Account<'info, Donation>,
    user: Signer<'info>,
    system_program: Program<'info, System>,
}

pub fn initialize(ctx: Context<Initialize>) -> Result<(), ProgramError> {
    let blood_donation_account = &mut ctx.accounts.blood_donation_account;
    blood_donation_account.is_initialized = true;
    // Other initialization logic
    Ok(())
}

