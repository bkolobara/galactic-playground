# Galactic Playground Architecture: A Visual UI Builder for Soroban Smart Contracts

## Overview

The Galactic Playground is a drag and drop UI builder, specifically made for building user
interfaces on top of [Soroban Smart Contracts][1]. This document provides a high level
overview on all the components of the platform.

[1]: https://stellar.org/soroban

## How it works

The Galactic Playground is split into two modes. First there is the "Builder Mode", a drag
and drop design interface that allows users to build a UI from different components, like
buttons, text and check boxes. They can define triggers and bind them to specific UI
elements. Soroban contracts expose a limited and well-specified interface, and this allows
us to provide UI components that match this interface. For example, a button trigger could
only be enabled if a linked address field and numerical input box are filled out, basically
defining a dependency relationship. And the button could then be bound to a Soroban
contract function call, using the address and input boxes as arguments. The "Builder Mode"
allows users to define all these rules and behaviors. Once the interface is ready, the
user can publish it under a name.

All published UIs would be available at the `/app/<name>` URL. There, anyone can connect
their wallet and interact with the contract through it.

There is also a backend component to the system, that provides a database for saving
drafts of UIs, while they are not ready yet. The backend is also a bridge between the
frontend and the Stellar RPC server.

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                       Frontend (TypeScript)                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│    ┌──────────────────────┐         ┌──────────────────────┐    │
│    │                      │ publish │                      │    │
│    │   Design Interface   │────────▶│   Runtime Interface  │    │
│    │   (Builder Mode)     │         │   (Published App)    │    │
│    │                      │         │                      │    │
│    └──────────────────────┘         └──────────────────────┘    │
│             │                                ▲                  │
│             │ Save Config                    │ Load Config      │
│             │                                │                  │
└─────────────┼────────────────────────────────┼──────────────────┘
              │                                │
              │                                │
              ▼                                │
┌─────────────────────────────────────────────────────────────────┐
│                      Backend Services (Rust)                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────┐   │
│  │                  │  │                  │  │              │   │
│  │   API Gateway    │  │  Event Manager   │  │   Storage    │   │
│  │                  │  │   (WebSocket)    │  │   (SQLite)   │   │
│  │                  │  │                  │  │              │   │
│  └──────────────────┘  └──────────────────┘  └──────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
                                   │
                                   │ RPC
                                   ▼
                    ┌──────────────────────────────┐
                    │                              │
                    │   Stellar RPC Server         │
                    │   (Soroban Interface)        │
                    │                              │
                    └──────────────────────────────┘
                                   │
                                   ▼
                    ┌──────────────────────────────┐
                    │                              │
                    │   Stellar Network            │
                    │   (Smart Contracts)          │
                    │                              │
                    └──────────────────────────────┘
```

### 1. Frontend Application (TypeScript/React)

#### 1.1 Design Interface (Builder Mode)

The "Builder Mode" interface consists of a Component Palette on the left. A component can
be selected and dropped into the middle canvas.

The Canvas has a grid structure and allows for positioning and resizing of different UI
elements. It's basically a way to define the whole final UI layout.

Once a component is placed into the canvas and selected, a Properties panel is shown that
allows you to define dependencies and triggers bound to that component.

There is also a Contract Inspector panel at the bottom, showing all available functions
inside the contract.

```
┌────────────────────────────────────────────────────────────┐
│                     Builder Interface                      │
├────────────────────────────────────────────────────────────┤
│                                                            │
│   ┌──────────────┐  ┌──────────────┐  ┌─────────────────┐  │
│   │              │  │              │  │                 │  │
│   │  Component   │  │    Canvas    │  │   Properties    │  │
│   │   Palette    │  │ (Grid-based) │  │     Panel       │  │
│   │              │  │              │  │                 │  │
│   │ • Input      │  │  ┌────────┐  │  │ Component: Btn  │  │
│   │ • Button     │  │  │   BTN  │  │  │ Function: inc() │  │
│   │ • Layout     │  │  └────────┘  │  │ Label: "Add"    │  │
│   │ • Wallet     │  │  ┌────────┐  │  │ Dependencies:   │  │
│   │ • Events     │  │  │   NUM  │  │  │  - wallet       │  │
│   │              │  │  └────────┘  │  │  - amount       │  │
│   └──────────────┘  └──────────────┘  └─────────────────┘  │
│                                                            │
│   ┌─────────────────────────────────────────────-───────┐  │
│   │              Contract Inspector                     │  │
│   │  Contract: CDLZFC3SYJYDZT7K67VZ75HP...              │  │
│   │  Functions: increment(u32), decrement(u32)          │  │
│   │  Events: COUNT, TRANSFER                            │  │
│   └─────────────────────────────────────────────-───────┘  │
└────────────────────────────────────────────────────────────┘
```

The final output of the builder phase is a JSON file defining all the positioning of the
UI elements and their behaviors.

#### 1.2 Runtime Interface (Published Apps)

The runtime interprets the JSON configurations to render the final interface to users.

```
┌─────────────────────────────────────────────────────┐
│              Runtime Execution Flow                 │
├─────────────────────────────────────────────────────┤
│                                                     │
│  1. Load JSON Config                                │
│     ▼                                               │
│  ┌──────────────────────────────────────┐           │
│  │ {                                    │           │
│  │   "layout": [...],                   │           │
│  │   "components": [...],               │           │
│  │   "bindings": [...],                 │           │
│  │   "events": [...],                   │           │
│  │   "customJS": "..."                  │           │
│  └──────────────────────────────────────┘           │
│                                                     │
│  2. Initialize Components                           │
│     ▼                                               │
│  ┌──────────────────────────────────────┐           │
│  │  • Parse component definitions       │           │
│  │  • Create React elements             │           │
│  │  • Bind event handlers               │           │
│  │  • Setup state management            │           │
│  └──────────────────────────────────────┘           │
│                                                     │
│  4. Render Interactive UI                           │
│     ▼                                               │
│  ┌──────────────────────────────────────┐           │
│  │         User Interface Ready         │           │
│  └──────────────────────────────────────┘           │
└─────────────────────────────────────────────────────┘
```

### 2. Backend (Rust)

The backend is a Rust Axum application providing endpoints for creating, updating and
publishing projects. It also provides proxy-endpoints for the frontend to talk to the
Stellar RPC server.

```rust
// Login endpoints
POST   /api/login
POST   /api/logout

// Project endpoints
GET    /api/projects                 // Retrieve all projects
POST   /api/projects                 // Create new project
GET    /api/projects/:id             // Retrieve specific project
PUT    /api/projects/:id             // Update project
POST   /api/projects/:id/publish     // Publish to /app/:name

// Contract endpoints
GET    /api/contract/:address/spec  // Get contract specification
POST   /api/contract/:address/call  // Call a contract function
GET    /api/contract/:address/state?query=<state>  // Fetch some state from a contract

// Events endpoint
WS     /ws/contract/:address/events // Event streaming
```

## Feasibility study (KALE contract)

To assess the feasibility of the project, I have built a prototype UI on top of the KALE
contract. The code for it can be found in [`/feasibility/kale`](/feasibility/kale/). It
consists of:

1. [docs](/feasibility/kale/docs/) - Documentation and learnings specific to the used
   Albedo wallet and KALE contract.
2. [Rust Server](/feasibility/kale/src/) - A Rust server built specifically to interact
   with the KALE contract through Soroban's RPC endpoint.
3. [Frontend UI](/feasibility/kale/frontend/) - The UI is specific to the KALE contract
   and meant to showcase some common patterns that come up when interacting with a Soroban
   contract.

### The KALE UI

This section contains the user flow of interacting with the KALE contract through the
prototype UI.

#### 1. Wallet connection

When the application starts up, the user is first presented with an option to connect a
wallet. All other functionality depends on having a wallet connection, so no other UI
elements are shown before. I believe that having the possibility of expressing what the
users see based on a state condition is very important and should be part of the UI
toolkit.

![KALE UI Screenshot](/feasibility/kale/assets/1_connect_wallet.png)

#### 2. Albedo Pop-up

Once the user clicks to connect a wallet, a new pop-up shows up. Allowing them to connect
the wallet. In this case they can bring their own wallet. The only information read from
the wallet at this point is the address and Lumens amount. Integration with wallets should
be a pre-built feature of the playground and all other UIs should have the possibility to
easily connect a wallet.

![KALE UI Screenshot](/feasibility/kale/assets/2_connect_wallet_popup.png)

#### 3. Stellar network state check

Once the user connects the wallet, the app checks directly if they have enough funding and
if there is a trustline towards the KALE contract. Both are pre-requirements to interact
with the contract and need to be setisfied. The user has right away the option to fund
their `testnet` account with Friendbot and doesn't need to leave the app to satisfy that
requirement.

![KALE UI Screenshot](/feasibility/kale/assets/3_fund_with_friendbot.png)

#### 4. Add `trustline`

Many contracts interact with the Stellar network and the playground will need to provide
functionality to query the network state itself, like balances and trustlines. The UI also
needs to offer a way of interacting with the Stellar network, like the possibility of
adding a trustline. In the case of the KALE UI, a trustline request needs to be approved
by the user through their wallet.

![KALE UI Screenshot](/feasibility/kale/assets/4_add_trustline.png)

![KALE UI Screenshot](/feasibility/kale/assets/5_add_trustline_popup.png)

#### 5. Invoke a Soroban function

The KALE UI can also invoke directly Soroban functions by prompting the user to sign the
call with their wallet. In this example, we are calling the `plant` function in the KALE
contract.

![KALE UI Screenshot](/feasibility/kale/assets/6_plant.png)

![KALE UI Screenshot](/feasibility/kale/assets/7_plant_popup.png)

#### 5. Fetch Soroban contract state

One important part of interacting with a contract is the ability to fetch the contract
state. The KALE contract is able to fetch stuff like the current field number and all
the kale that was planted by the user or anyone else in the last few fields. Having access
to that state allows us to nicely display each kale planted, their current quality and who
it belongs to.

![KALE UI Screenshot](/feasibility/kale/assets/8_work.png)

![KALE UI Screenshot](/feasibility/kale/assets/9_work_popup.png)

![KALE UI Screenshot](/feasibility/kale/assets/10_improve_work.png)

![KALE UI Screenshot](/feasibility/kale/assets/11_harvest.png)

#### Conclusion

The KALE contract lives on the Stellar blockchain but can be interacted with through
different approaches, like [an existing web interface](https://kalefarm.xyz) or even a
[GPU farming interface](https://github.com/FredericRezeau/kale-miner).

The KALE UI is just another interface to interact with the KALE contract itself. Building
it up from scratch was a significant effort and I believe that many of the building blocks
can be reused for providing UIs for other contracts. This was the main motivation for
building the Galactic Playground. It would be a toolbox of common UI patterns bound to
contract elements, like functions or state. This should significantly lower the burden of
building a UI from a contract.
