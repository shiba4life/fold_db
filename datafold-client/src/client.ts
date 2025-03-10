import axios, { AxiosInstance, AxiosRequestConfig } from 'axios';
import { 
  DataFoldClientConfig, 
  SchemaConfig, 
  Operation, 
  QueryResponse, 
  MutationResponse,
  NetworkConfig,
  NetworkStatus,
  NodeInfo,
  DataFoldError
} from './types';

/**
 * DataFold client for interacting with a datafold node
 */
export class DataFoldClient {
  private client: AxiosInstance;
  private config: DataFoldClientConfig;

  /**
   * Create a new DataFold client
   * @param config Client configuration
   */
  constructor(config: DataFoldClientConfig) {
    this.config = config;

    const axiosConfig: AxiosRequestConfig = {
      baseURL: config.baseUrl,
      timeout: config.timeout || 10000,
      headers: {
        'Content-Type': 'application/json',
      },
    };

    // If using Unix socket
    if (config.socketPath) {
      axiosConfig.socketPath = config.socketPath;
    }

    this.client = axios.create(axiosConfig);

    // Add auth headers if provided
    if (config.auth) {
      this.client.interceptors.request.use((config) => {
        config.headers = config.headers || {};
        config.headers['X-Public-Key'] = this.config.auth?.public_key;
        
        // In a real implementation, we would sign the request with the private key
        // This is a placeholder for the actual signature implementation
        if (this.config.auth?.private_key) {
          const timestamp = Date.now().toString();
          config.headers['X-Timestamp'] = timestamp;
          config.headers['X-Signature'] = `signature-placeholder-${timestamp}`;
        }
        
        return config;
      });
    }
  }

  /**
   * Get a list of all schemas
   * @returns Promise resolving to an array of schema names
   */
  async listSchemas(): Promise<string[]> {
    try {
      const response = await this.client.get('/api/schemas');
      return response.data.schemas;
    } catch (error) {
      this.handleError(error);
      return [];
    }
  }

  /**
   * Create or update a schema
   * @param schema Schema configuration
   * @returns Promise resolving to success status
   */
  async createSchema(schema: SchemaConfig): Promise<boolean> {
    try {
      const response = await this.client.post('/api/schema', schema);
      return response.data.success;
    } catch (error) {
      this.handleError(error);
      return false;
    }
  }

  /**
   * Delete a schema
   * @param name Schema name
   * @returns Promise resolving to success status
   */
  async deleteSchema(name: string): Promise<boolean> {
    try {
      const response = await this.client.delete(`/api/schema/${name}`);
      return response.data.success;
    } catch (error) {
      this.handleError(error);
      return false;
    }
  }

  /**
   * Execute a query operation
   * @param query Query operation
   * @returns Promise resolving to query results
   */
  async query<T = any>(query: Operation): Promise<QueryResponse> {
    if (query.type !== 'query') {
      throw new Error('Operation must be a query');
    }

    try {
      const response = await this.client.post('/api/execute', {
        operation: JSON.stringify(query)
      });
      
      return {
        results: response.data.results as T[],
        count: response.data.count
      };
    } catch (error) {
      this.handleError(error);
      return { results: [], count: 0 };
    }
  }

  /**
   * Execute a mutation operation
   * @param mutation Mutation operation
   * @returns Promise resolving to mutation response
   */
  async mutate(mutation: Operation): Promise<MutationResponse> {
    if (mutation.type !== 'mutation') {
      throw new Error('Operation must be a mutation');
    }

    try {
      const response = await this.client.post('/api/execute', {
        operation: JSON.stringify(mutation)
      });
      
      return {
        success: response.data.success,
        affected_count: response.data.affected_count || 0,
        error: response.data.error
      };
    } catch (error) {
      this.handleError(error);
      return { success: false, affected_count: 0, error: 'Request failed' };
    }
  }

  /**
   * Create a record
   * @param schema Schema name
   * @param data Record data
   * @returns Promise resolving to mutation response
   */
  async create(schema: string, data: Record<string, any>): Promise<MutationResponse> {
    const mutation = {
      type: 'mutation',
      schema,
      operation: 'create',
      data
    } as Operation;

    return this.mutate(mutation);
  }

  /**
   * Update records matching a filter
   * @param schema Schema name
   * @param filter Query filter
   * @param data Update data
   * @returns Promise resolving to mutation response
   */
  async update(schema: string, filter: Record<string, any>, data: Record<string, any>): Promise<MutationResponse> {
    const mutation = {
      type: 'mutation',
      schema,
      operation: 'update',
      filter,
      data
    } as Operation;

    return this.mutate(mutation);
  }

  /**
   * Delete records matching a filter
   * @param schema Schema name
   * @param filter Query filter
   * @returns Promise resolving to mutation response
   */
  async delete(schema: string, filter: Record<string, any>): Promise<MutationResponse> {
    const mutation = {
      type: 'mutation',
      schema,
      operation: 'delete',
      filter
    } as Operation;

    return this.mutate(mutation);
  }

  /**
   * Find records matching a filter
   * @param schema Schema name
   * @param fields Fields to return
   * @param filter Query filter
   * @returns Promise resolving to query results
   */
  async find<T = any>(schema: string, fields: string[], filter: Record<string, any> | null = null): Promise<T[]> {
    const query = {
      type: 'query',
      schema,
      fields,
      filter
    } as Operation;

    const response = await this.query<T>(query);
    return response.results;
  }

  /**
   * Find a single record matching a filter
   * @param schema Schema name
   * @param fields Fields to return
   * @param filter Query filter
   * @returns Promise resolving to a single record or null
   */
  async findOne<T = any>(schema: string, fields: string[], filter: Record<string, any>): Promise<T | null> {
    const results = await this.find<T>(schema, fields, filter);
    return results.length > 0 ? results[0] : null;
  }

  /**
   * Get network status
   * @returns Promise resolving to network status
   */
  async getNetworkStatus(): Promise<NetworkStatus> {
    try {
      const response = await this.client.get('/api/network/status');
      return response.data;
    } catch (error) {
      this.handleError(error);
      return {
        running: false,
        node_count: 0,
        connection_count: 0
      };
    }
  }

  /**
   * Initialize the network
   * @param config Network configuration
   * @returns Promise resolving to success status
   */
  async initNetwork(config: NetworkConfig): Promise<boolean> {
    try {
      const response = await this.client.post('/api/network/init', config);
      return response.data.success;
    } catch (error) {
      this.handleError(error);
      return false;
    }
  }

  /**
   * Start the network
   * @returns Promise resolving to success status
   */
  async startNetwork(): Promise<boolean> {
    try {
      const response = await this.client.post('/api/network/start', {});
      return response.data.success;
    } catch (error) {
      this.handleError(error);
      return false;
    }
  }

  /**
   * Stop the network
   * @returns Promise resolving to success status
   */
  async stopNetwork(): Promise<boolean> {
    try {
      const response = await this.client.post('/api/network/stop', {});
      return response.data.success;
    } catch (error) {
      this.handleError(error);
      return false;
    }
  }

  /**
   * Discover nodes on the network
   * @returns Promise resolving to success status
   */
  async discoverNodes(): Promise<boolean> {
    try {
      const response = await this.client.post('/api/network/discover', {});
      return response.data.success;
    } catch (error) {
      this.handleError(error);
      return false;
    }
  }

  /**
   * Connect to a node
   * @param address Node address
   * @param port Node port
   * @returns Promise resolving to success status
   */
  async connectToNode(address: string, port: number): Promise<boolean> {
    try {
      const response = await this.client.post('/api/network/connect', {
        address,
        port
      });
      return response.data.success;
    } catch (error) {
      this.handleError(error);
      return false;
    }
  }

  /**
   * List connected nodes
   * @returns Promise resolving to an array of node info
   */
  async listNodes(): Promise<NodeInfo[]> {
    try {
      const response = await this.client.get('/api/network/nodes');
      return response.data.nodes;
    } catch (error) {
      this.handleError(error);
      return [];
    }
  }

  /**
   * Register a public key for authentication
   * @param publicKey Public key
   * @param signature Signature proving ownership of the key
   * @returns Promise resolving to success status
   */
  async registerKey(publicKey: string, signature: string): Promise<boolean> {
    try {
      const response = await this.client.post('/api/auth/register', {
        public_key: publicKey,
        signature
      });
      return response.data.success;
    } catch (error) {
      this.handleError(error);
      return false;
    }
  }

  /**
   * Handle errors from the API
   * @param error Error object
   * @throws Formatted error
   */
  private handleError(error: any): never {
    if (axios.isAxiosError(error) && error.response?.data) {
      const errorData = error.response.data;
      const errorObj: DataFoldError = {
        code: errorData.code || 'UNKNOWN_ERROR',
        message: errorData.message || error.message || 'Unknown error',
        details: errorData.details
      };
      throw errorObj;
    }
    
    throw error;
  }
}
