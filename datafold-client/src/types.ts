/**
 * Types for the datafold client
 */

// Schema related types
export interface PermissionPolicy {
  read_policy?: {
    NoRequirement?: null;
    Distance?: number;
  };
  write_policy?: {
    NoRequirement?: null;
    Distance?: number;
  };
  explicit_read_policy?: {
    counts_by_pub_key?: Record<string, number>;
  } | null;
  explicit_write_policy?: {
    counts_by_pub_key?: Record<string, number>;
  } | null;
}

export interface PaymentConfig {
  base_multiplier: number;
  trust_distance_scaling?: {
    None?: null;
    Linear?: {
      slope: number;
      intercept: number;
      min_factor: number;
    };
  };
  min_payment?: number | null;
}

export interface FieldConfig {
  permission_policy: PermissionPolicy;
  payment_config: PaymentConfig;
  field_mappers: Record<string, any>;
}

export interface SchemaConfig {
  name: string;
  fields: Record<string, FieldConfig>;
  payment_config?: {
    base_multiplier: number;
    min_payment_threshold?: number;
  };
}

// Query related types
export interface QueryFilter {
  [key: string]: any;
}

export interface QueryOperation {
  type: 'query';
  schema: string;
  fields: string[];
  filter: QueryFilter | null;
}

// Mutation related types
export interface CreateMutation {
  type: 'mutation';
  schema: string;
  operation: 'create';
  data: Record<string, any>;
}

export interface UpdateMutation {
  type: 'mutation';
  schema: string;
  operation: 'update';
  filter: QueryFilter;
  data: Record<string, any>;
}

export interface DeleteMutation {
  type: 'mutation';
  schema: string;
  operation: 'delete';
  filter: QueryFilter;
}

export type MutationOperation = CreateMutation | UpdateMutation | DeleteMutation;

export type Operation = QueryOperation | MutationOperation;

// Response types
export interface QueryResponse {
  results: any[];
  count: number;
}

export interface MutationResponse {
  success: boolean;
  affected_count: number;
  error?: string;
}

// Network related types
export interface NetworkConfig {
  listen_addr: string;
  port: number;
  public_key?: string;
  private_key?: string;
}

export interface NodeInfo {
  id: string;
  addr: string;
  port: number;
  public_key: string;
  status: 'connected' | 'disconnected' | 'pending';
}

export interface NetworkStatus {
  running: boolean;
  node_count: number;
  connection_count: number;
  local_node_id?: string;
}

// Auth related types
export interface AuthConfig {
  public_key: string;
  private_key: string;
  trust_level?: number;
}

// Client configuration
export interface DataFoldClientConfig {
  baseUrl: string;
  auth?: AuthConfig;
  timeout?: number;
  socketPath?: string;
}

// Error types
export interface DataFoldError {
  code: string;
  message: string;
  details?: any;
}
