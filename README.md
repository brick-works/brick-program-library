# brick-program-library

Brickworks is paving and building bricks on Solana. Check out our programs!

Note: Brick programs **are subject to change**, are in active development. This code is unaudited. Use at your own risk.

## Key dependencies

- solana-cli: 1.14.25
- anchor-cli: 0.28.0

## Run program tests

1. git clone https://github.com/brick-works/brick-program-library
2. anchor build
3. anchor test

## Overview

- Marketplace-manager: Payment solution with marketplace mechanics: fees enforcement, rewards and token/cNFT dispenser.
- Product-manager: Payment solution to use as index to get direct & escrow payment events.
- Tender: Designed for the public bidding of public projects, offering the community the ability to submit project proposals.
- User-manager: 🥸?

## Marketplace customization

Marketplace creators have the freedom to customize their platforms based on their specific requirements. 

The following customizable features are available:
  
- **Manage Access**: Marketplaces can opt for a permissionless model, allowing anyone to sell on the platform without the need for verification. Alternatively, a token gate or KYC process can be implemented for controlled access.

- **Rewards or Cashback**: Marketplaces can incentivize users by offering rewards or cashback for their purchases on the platform, enhancing user engagement and loyalty.

- **Transaction Fees**: The marketplace has the flexibility to set transaction fees for facilitating transactions between buyers and sellers. This allows marketplaces to generate revenue from the platform operations.
  
- **Secondary Market Support**: Marketplaces can decide whether products or services can be resold, thereby creating a secondary market for items. When users register a purchase, a token can be minted, granting them access to the specific product or service.
