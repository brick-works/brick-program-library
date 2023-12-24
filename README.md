# brick-program-library

BrickWorks is paving and building bricks on Solana. Check out our programs!

Note: Brick programs **are subject to change**, are in active development. This code is unaudited. Use at your own risk.

## Key dependencies

- solana-cli 1.17.13
- anchor: 0.29.0

## Overview

- Marketplace-manager: Payment solution with marketplace mechanics: fees enforcement, rewards and token/cNFT dispenser.
- Product-manager: Payment solution to use as index to get direct & escrow payment events.
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
4. anchor deploy
5. close local validator
6. anchor test
