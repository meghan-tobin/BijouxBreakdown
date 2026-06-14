# BijouxBreakdown
## Problem Statement

Artists and hobbyists who create physical goods often rely on spreadsheets, handwritten notes, or multiple disconnected tools to track materials, time spent, and product pricing. These methods can be time-consuming, difficult to maintain, and make it challenging to understand the true cost and profitability of each piece.

Additionally, creators often organize their work into collections or product lines, but existing tools do not provide an intuitive way to manage collections, track material variations, or compare similar products. As a result, many makers struggle to accurately price their work and maintain an organized catalog of their creations.

## Proposed Solution

Create a simple and intuitive platform that helps artists and makers manage their materials, organize products into collections, and automatically calculate the cost and pricing of their creations. The platform should reduce manual tracking while providing clear visibility into material costs, labor, and profitability.

## Target Users

#### Primary Users:

Independent artists and hobbyists who create physical products
Small-scale makers selling through craft fairs, social media, or online marketplaces

#### Examples:

Jewelry makers
Potters and ceramic artists
Knitters and crocheters 
Woodworkers
Candle/soap/wax creators
Resin artists
Other handmade goods creators

## Goals
- Enable creators to build reusable supply configurations for different types of work.
- Use those configurations to define individual creations and track exactly what materials go into each one.
- Automatically calculate the production cost of each creation based on material usage.
- Optionally track time costs per configuration to accurately represent labor in pricing.
- Allow creators to set a selling price per creation and surface the resulting profit margin.
- Organize creations into groups or collections for easy catalog management.
- Provide a clear, at-a-glance board view of all creations and their costs.


## Key Requirements
- Creators can define named supply configurations with individual materials (name, cost, quantity, unit)
- Multiple configurations can exist side by side (e.g. one for earrings, one for necklaces)
- Individual creations are built from a saved configuration, with per-supply usage amounts entered by the creator
- Material cost is calculated automatically based on usage
- Creations appear as cards on a free-form board that can be dragged, resized, and organized
- Creations can be grouped into named collections on the board
- All data persists locally in the browser across sessions

## Non-Goals
- Cloud sync or multi-device access
- Inventory tracking or stock level management
- Direct integration with suppliers or purchasing platforms
- E-commerce or storefront functionality
- Mobile app (desktop browser only for now)
- Multi-user or team collaboration

## Success Metrics
- A creator can go from a blank slate to a fully costed item in under 5 minutes
- Cost calculations are accurate and update immediately when usage amounts change
- The board gives a clear at-a-glance overview of all creations and what they cost to make
- Data survives page refreshes and browser restarts without any manual export
- Users frequently return to the application

