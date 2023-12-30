# brick-program-library

Just building bricks and experimenting on Solana. Check out the programs!

## Dependencies

- solana-cli 1.17.13
- anchor: 0.29.0

## Overview

- Marketplace-manager: Payment solution with marketplace mechanics: fees enforcement and rewards.
- Product-manager: Simple payment solution to perform direct & escrow payment events.
- Tender: Designed for the bidding of public projects, offering the community the ability to submit project proposals and deposits.

## Marketplace customization

Marketplace creators have the freedom to customize their platforms based on their specific requirements. 

The following customizable features are available:
  
- **Manage Access**: Marketplaces can opt for a permissionless model, allowing anyone to sell on the platform without the need for verification. Alternatively, a token gate or KYC process can be implemented for controlled access.

- **Rewards or Cashback**: Marketplaces can incentivize users by offering rewards or cashback for their purchases on the platform, enhancing user engagement and loyalty.

- **Transaction Fees**: The marketplace has the flexibility to set transaction fees for facilitating transactions between buyers and sellers. This allows marketplaces to generate revenue from the platform operations.

## Run program tests

Note: preferably run the tests individually by modifying the script on Anchor.toml

1. git clone https://github.com/brick-works/brick-program-library
2. anchor build
3. solana-test-validator
4. anchor deploy and close local validator
5. anchor test
