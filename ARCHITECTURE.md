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
