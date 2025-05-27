# Pure Unified Operations Architecture Diagram

## High-Level Architecture Overview

```mermaid
graph TD
    A[DataFold System<br/>Pure Unified Architecture] --> B[FoldDB<br/>Main Entry Point]
    B --> C[DbOperations<br/>Unified Database Interface]
    
    C --> D[Metadata Operations]
    C --> E[Transform Operations]
    C --> F[Schema Operations]
    C --> G[Orchestrator Operations]
    C --> H[Atom Operations]
    C --> I[Permission Operations]
    
    D --> J[Sled Database<br/>Physical Storage]
    E --> J
    F --> J
    G --> J
    H --> J
    I --> J
    
    style A fill:#e1f5fe
    style B fill:#f3e5f5
    style C fill:#fff3e0
    style J fill:#e8f5e8
```

## Component Architecture Detail

```mermaid
graph TB
    subgraph FoldDB["FoldDB (Main Application Layer)"]
        SC[SchemaCore<br/>new(DbOperations)]
        TM[TransformManager<br/>new(DbOperations)]
        AM[AtomManager<br/>new(DbOperations)]
    end
    
    subgraph DbOps["DbOperations (Single Source of Truth)"]
        subgraph SchemaOps["Schema Operations"]
            SSS[store_schema_state]
            GSS[get_schema_state]
            LSS[list_schemas_by_state]
            DSS[delete_schema_state]
        end
        
        subgraph TransformOps["Transform Operations"]
            ST[store_transform]
            GT[get_transform]
            LT[list_transforms]
            DT[delete_transform]
            STM[store_transform_mapping]
        end
        
        subgraph MetadataOps["Metadata Operations"]
            SI[store_item]
            GI[get_item]
            DI[delete_item]
            LK[list_keys]
        end
        
        subgraph OrchestratorOps["Orchestrator Operations"]
            SOS[store_orchestrator_state]
            GOS[get_orchestrator_state]
            LOK[list_orchestrator_keys]
        end
        
        subgraph AtomOps["Atom Operations"]
            CA[create_atom]
            UAR[update_atom_ref]
            UARC[update_atom_ref_*]
        end
        
        subgraph PermOps["Permission Operations"]
            GSP[get_schema_permissions]
            SSP[set_schema_permissions]
        end
    end
    
    subgraph SledDB["Sled Database (Physical Storage)"]
        MT[metadata tree]
        TT[transforms tree]
        ST2[schemas tree]
        OT[orchestrator tree]
        PT[permissions tree]
        AT[atoms tree]
        RT[ranges tree]
    end
    
    SC --> DbOps
    TM --> DbOps
    AM --> DbOps
    
    SchemaOps --> SledDB
    TransformOps --> SledDB
    MetadataOps --> SledDB
    OrchestratorOps --> SledDB
    AtomOps --> SledDB
    PermOps --> SledDB
    
    style FoldDB fill:#e1f5fe
    style DbOps fill:#fff3e0
    style SledDB fill:#e8f5e8
    style SC fill:#f3e5f5
    style TM fill:#f3e5f5
    style AM fill:#f3e5f5
```

## Data Flow Architecture

```mermaid
flowchart TD
    subgraph AppLayer["Application Layer"]
        UR[User Request] --> FM[FoldDB.method]
        FM --> CM[Component.method]
    end
    
    subgraph BizLayer["Business Logic Layer"]
        subgraph Components["Components"]
            SC2[SchemaCore<br/>• Schema logic<br/>• Validation<br/>• State mgmt]
            TM2[TransformManager<br/>• Transform validation<br/>• Registration<br/>• Execution]
            AM2[AtomManager<br/>• Atom logic<br/>• Reference mgmt]
        end
        
        Note1[ALL OPERATIONS GO THROUGH<br/>UNIFIED INTERFACE]
    end
    
    subgraph DbLayer["Database Abstraction Layer"]
        DbOps2[DbOperations<br/>• Single point of database access<br/>• Consistent error handling<br/>• Transaction support (future)<br/>• Caching layer (future)<br/>• Monitoring & metrics (future)]
    end
    
    subgraph StorageLayer["Storage Layer"]
        SledDB2[Sled Database<br/>• Embedded key-value store<br/>• ACID transactions<br/>• Crash-safe persistence<br/>• Multiple trees for data organization]
    end
    
    CM --> Components
    SC2 --> DbOps2
    TM2 --> DbOps2
    AM2 --> DbOps2
    DbOps2 --> SledDB2
    
    style AppLayer fill:#e1f5fe
    style BizLayer fill:#f3e5f5
    style DbLayer fill:#fff3e0
    style StorageLayer fill:#e8f5e8
    style Note1 fill:#ffeb3b,color:#000
```

## Operation Flow Examples

### Schema Operations Flow
```mermaid
sequenceDiagram
    participant User
    participant FoldDB
    participant SchemaCore
    participant DbOps as DbOperations
    participant Sled as Sled Database
    
    User->>FoldDB: create_schema()
    FoldDB->>SchemaCore: create_schema()
    SchemaCore->>DbOps: store_schema(schema_id, schema_data)
    DbOps->>Sled: Store to database
    Sled-->>DbOps: Success
    DbOps-->>SchemaCore: Success
    SchemaCore-->>FoldDB: Success
    FoldDB-->>User: Schema created
```

### Transform Operations Flow
```mermaid
sequenceDiagram
    participant User
    participant FoldDB
    participant TM as TransformManager
    participant DbOps as DbOperations
    participant Sled as Sled Database
    
    User->>FoldDB: register_transform()
    FoldDB->>TM: register_transform()
    TM->>DbOps: store_transform(transform_id, transform_data)
    DbOps->>Sled: Store transform
    TM->>DbOps: store_transform_mapping(schema_id, transform_id)
    DbOps->>Sled: Store mapping
    Sled-->>DbOps: Success
    DbOps-->>TM: Success
    TM-->>FoldDB: Success
    FoldDB-->>User: Transform registered
```

### Atom Operations Flow
```mermaid
sequenceDiagram
    participant User
    participant FoldDB
    participant AM as AtomManager
    participant DbOps as DbOperations
    participant Sled as Sled Database
    
    User->>FoldDB: create_atom()
    FoldDB->>AM: create_atom()
    AM->>DbOps: store_atom(atom_id, atom_data)
    DbOps->>Sled: Store atom
    AM->>DbOps: update_atom_references(references)
    DbOps->>Sled: Update references
    Sled-->>DbOps: Success
    DbOps-->>AM: Success
    AM-->>FoldDB: Success
    FoldDB-->>User: Atom created
```

### Transform Registration Flow (Detailed)
```mermaid
flowchart TD
    UR[User Request] --> FDB[FoldDB.register_transform]
    FDB --> TM[TransformManager.register_transform]
    
    TM --> VT[Validate Transform]
    VT --> UMC[Update In-Memory Cache]
    UMC --> PTD[Persist to Database]
    
    PTD --> ST[DbOperations.store_transform]
    ST --> STM[DbOperations.store_transform_mapping]
    STM --> SDB[Sled DB transforms tree]
    
    SDB --> Success[Success Response]
    Success --> FDB
    FDB --> UR
    
    style UR fill:#e1f5fe
    style FDB fill:#f3e5f5
    style TM fill:#f3e5f5
    style ST fill:#fff3e0
    style STM fill:#fff3e0
    style SDB fill:#e8f5e8
```

## Key Architecture Principles

### 1. Single Source of Truth
```mermaid
flowchart LR
    subgraph BL["Business Logic"]
        SC[SchemaCore]
        TM[TransformManager]
        AM[AtomManager]
    end
    
    subgraph UI["Unified Interface"]
        DbOps[DbOperations]
    end
    
    subgraph PS["Physical Storage"]
        Sled[Sled Database]
    end
    
    SC --> DbOps
    TM --> DbOps
    AM --> DbOps
    DbOps --> Sled
    
    style BL fill:#f3e5f5
    style UI fill:#fff3e0
    style PS fill:#e8f5e8
```

### 2. No Fallback Paths
```mermaid
flowchart TD
    subgraph Old["❌ OLD (Hybrid)"]
        C1[Component] --> DO1{DbOperations?}
        DO1 -->|Available| S1[Success]
        DO1 -->|Unavailable| LS[Legacy Sled]
        LS --> FB[Fallback]
    end
    
    subgraph New["✅ NEW (Pure)"]
        C2[Component] --> DO2[DbOperations]
        DO2 --> S2[Success]
        DO2 --> E2[Error]
        E2 --> FF[Fail Fast]
    end
    
    style Old fill:#ffebee
    style New fill:#e8f5e8
    style C1 fill:#f3e5f5
    style C2 fill:#f3e5f5
    style DO2 fill:#fff3e0
```

### 3. Consistent Error Handling
```mermaid
flowchart TD
    Op[All Operations] --> Success{Success?}
    Success -->|Yes| RR[Return Result]
    Success -->|DbOperations Error| SE1[SchemaError]
    Success -->|No DbOperations| SE2["SchemaError('DbOperations not available')"]
    
    style Op fill:#e1f5fe
    style RR fill:#e8f5e8
    style SE1 fill:#ffebee
    style SE2 fill:#ffebee
```

### 4. Future-Ready Design
```mermaid
flowchart TD
    subgraph Current["Current Architecture"]
        C1[Component] --> DO1[DbOperations] --> S1[Sled]
    end
    
    subgraph Future["Future Architecture"]
        C2[Component] --> DO2[DbOperations]
        DO2 --> T[Transactions]
        DO2 --> CA[Caching]
        DO2 --> M[Monitoring]
        DO2 --> R[Replication]
        T --> S2[Sled]
        CA --> S2
        M --> S2
        R --> S2
    end
    
    style Current fill:#e1f5fe
    style Future fill:#e8f5e8
    style DO1 fill:#fff3e0
    style DO2 fill:#fff3e0
    style S1 fill:#e8f5e8
    style S2 fill:#e8f5e8
```

## Benefits of Pure Unified Architecture

### 1. **Simplicity**
- Single constructor per component
- Single database access path
- No conditional logic for fallbacks

### 2. **Consistency**
- All operations use same interface
- Predictable error handling
- Uniform behavior across components

### 3. **Maintainability**
- Clear dependency requirements
- Easier debugging and testing
- Reduced code complexity

### 4. **Extensibility**
- Ready for advanced features
- Clean foundation for transactions
- Unified caching and monitoring

This pure unified architecture provides a clean, maintainable, and extensible foundation for the DataFold system with no legacy fallback paths remaining.