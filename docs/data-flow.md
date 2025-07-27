# BeBytes Macro Data Flow

Data flow through the BeBytes derive macro.

## Overview


```mermaid
flowchart TD
    A[TokenStream Input] --> B[Parse DeriveInput]
    B --> C{Data Type?}
    C -->|Struct| D[Process Struct]
    C -->|Enum| E[Process Enum]
    
    D --> F[Extract Fields]
    F --> G[For Each Field]
    G --> H[Determine Field Type]
    H --> I[Generate Parsing Code]
    H --> J[Generate Writing Code]
    H --> K[Generate Validation]
    
    I --> L[Collect BE Parsing]
    I --> M[Collect LE Parsing]
    J --> N[Collect BE Writing]
    J --> O[Collect LE Writing]
    K --> P[Collect Limit Checks]
    
    L --> Q[Generate Trait Impl]
    M --> Q
    N --> Q
    O --> Q
    P --> Q
    
    Q --> R[TokenStream Output]
    
    E --> S[Check Flags Attribute]
    S --> T[Validate Enum Variants]
    T --> U[Generate Enum Impl]
    U --> R
```

## Struct Processing Pipeline

### 1. Initial Data Collection

```mermaid
flowchart LR
    A[FieldsNamed] --> B[StructContext]
    B --> C[field_limit_check: Vec]
    B --> D[errors: Vec]
    B --> E[field_parsing: Vec]
    B --> F[bit_sum: Vec]
    B --> G[field_writing: Vec]
    B --> H[named_fields: Vec]
    B --> I[endianness: Endianness]
```

### 2. Field Type Determination

```mermaid
flowchart TD
    A[Field + Attributes] --> B{Has #[bits] attr?}
    B -->|Yes| C[Parse Bit Size]
    B -->|No| D{Check Type}
    
    C --> E[FieldType::BitsField]
    
    D --> F{Is Primitive?}
    D --> G{Is Array?}
    D --> H{Is Vec?}
    D --> I{Is Option?}
    D --> J{Is Custom?}
    
    F -->|Yes| K[FieldType::PrimitiveType]
    G -->|Yes| L[FieldType::Array]
    H -->|Yes| M[FieldType::Vector]
    I -->|Yes| N[FieldType::OptionType]
    J -->|Yes| O[FieldType::CustomType]
    
    M --> P{Has FromField?}
    P -->|Yes| Q[Extract Field Path]
    P -->|No| R{Is Last Field?}
    R -->|No| S[Error: Vec not last]
```

### 3. Bit Field Processing

```mermaid
flowchart TD
    A[BitsField Input] --> B[Check Type Support]
    B --> C{Supported?}
    C -->|No| D[Error: Unsupported type]
    C -->|Yes| E[Generate Limit Check]
    
    E --> F[Calculate Mask]
    F --> G[Generate Parsing Code]
    G --> H{Single Byte?}
    
    H -->|Yes| I[Simple Bit Extract]
    H -->|No| J[Multi-byte Extract]
    
    I --> K[Update Bit Position]
    J --> K
    
    K --> L[Generate Writing Code]
    L --> M{Crosses Byte?}
    
    M -->|Yes| N[Split Across Bytes]
    M -->|No| O[Write to Current Byte]
    
    subgraph "Bit Position Tracking"
        P[bit_position += bits]
        Q[Check byte alignment]
        R[Reset on byte boundary]
    end
```

### 4. Endianness Generation

```mermaid
flowchart TD
    A[Field Processing] --> B{Endianness}
    B -->|Big Endian| C[BE Processing]
    B -->|Little Endian| D[LE Processing]
    
    C --> E[from_be_bytes calls]
    C --> F[to_be_bytes calls]
    
    D --> G[from_le_bytes calls]
    D --> H[to_le_bytes calls]
    
    subgraph "Shared Processing"
        I[field_limit_check]
        J[bit_sum calculation]
        K[field validation]
    end
    
    E --> L[BE field_parsing]
    F --> M[BE field_writing]
    
    G --> N[LE field_parsing]
    H --> O[LE field_writing]
```

## Data Structures Through the Pipeline

### Input Structure
```rust
#[derive(BeBytes)]
struct Example {
    #[bits(4)]
    flags: u8,
    #[bits(12)]
    value: u16,
    count: u8,
    #[FromField(count)]
    data: Vec<u8>,
}
```

### Processing Flow

```mermaid
flowchart TD
    A[Example Struct] --> B[Field: flags]
    A --> C[Field: value]
    A --> D[Field: count]
    A --> E[Field: data]
    
    B --> F["FieldType::BitsField(4)"]
    C --> G["FieldType::BitsField(12)"]
    D --> H["FieldType::PrimitiveType"]
    E --> I["FieldType::Vector(None, Some(['count']))"]
    
    F --> J[flags parsing: extract 4 bits]
    G --> K[value parsing: extract 12 bits]
    H --> L[count parsing: read u8]
    I --> M[data parsing: read count bytes]
    
    J --> N[Bit position: 0→4]
    K --> O[Bit position: 4→16]
    O --> P[Byte alignment check]
    
    subgraph "Generated BE Parsing"
        Q["let flags = extract_bits(bytes, 0, 4);"]
        R["let value = extract_bits(bytes, 4, 12);"]
        S["let count = u8::from_be_bytes(...);"]
        T["let data = Vec::from(&bytes[..count]);"]
    end
```

## Error Handling Flow

```mermaid
flowchart TD
    A[Field Processing] --> B{Validation}
    B --> C{Bits Complete Byte?}
    C -->|No| D[Error: Incomplete byte]
    
    B --> E{Vec Position Valid?}
    E -->|No| F[Error: Vec not last]
    
    B --> G{Type Supported?}
    G -->|No| H[Error: Unsupported type]
    
    B --> I{Buffer Sufficient?}
    I -->|No| J[Runtime Error: Buffer too small]
    
    D --> K[Compile Error]
    F --> K
    H --> K
    
    J --> L[Runtime Error Result]
    
    K --> M[TokenStream with compile_error!]
    L --> N[Result::Err(BeBytesError)]
```

## Generated Code Structure

```mermaid
flowchart TD
    A[Generated Implementation] --> B[impl BeBytes for Type]
    
    B --> C[field_size()]
    B --> D[try_from_be_bytes()]
    B --> E[to_be_bytes()]
    B --> F[try_from_le_bytes()]
    B --> G[to_le_bytes()]
    
    C --> H[Sum all field sizes]
    
    D --> I[Buffer validation]
    I --> J[Field parsing loop]
    J --> K[Construct Self]
    
    E --> L[Capacity allocation]
    L --> M[Field writing loop]
    M --> N[Return Vec]
    
    subgraph "Additional Generated"
        O[impl Type::new()]
        P[Field limit checks]
    end
```

## Bit Manipulation Details

### Reading Bits

```mermaid
flowchart TD
    A[Read N bits at position P] --> B{Fits in byte?}
    B -->|Yes| C[Single byte read]
    B -->|No| D[Multi-byte read]
    
    C --> E["mask = (1 << N) - 1"]
    C --> F["shift = 8 - P - N"]
    C --> G["value = (byte >> shift) & mask"]
    
    D --> H[Read first byte partial]
    D --> I[Read middle bytes full]
    D --> J[Read last byte partial]
    
    H --> K[Combine with shifts]
    I --> K
    J --> K
    
    K --> L[Final value]
```

### Writing Bits

```mermaid
flowchart TD
    A[Write N bits at position P] --> B{Fits in byte?}
    B -->|Yes| C[Single byte write]
    B -->|No| D[Multi-byte write]
    
    C --> E["mask = (1 << N) - 1"]
    C --> F["shift = 8 - P - N"]
    C --> G["byte |= (value & mask) << shift"]
    
    D --> H[Split value into parts]
    H --> I[Write first byte partial]
    H --> J[Write middle bytes full]
    H --> K[Write last byte partial]
    
    I --> L[Update byte buffer]
    J --> L
    K --> L
```

## Vector Field Processing

```mermaid
flowchart TD
    A[Vector Field] --> B{Has size constraint?}
    B -->|FromField| C[Dynamic size from field]
    B -->|With| D[Static size]
    B -->|None| E{Is last field?}
    
    C --> F[Generate field accessor]
    F --> G["let size = self.field.subfield;"]
    
    D --> H[Use literal size]
    
    E -->|Yes| I[Consume remaining bytes]
    E -->|No| J[Compile error]
    
    G --> K[Read size bytes]
    H --> K
    I --> L[Read all remaining]
    
    K --> M[Construct Vec]
    L --> M
```

