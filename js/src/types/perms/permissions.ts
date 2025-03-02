export interface PermissionCheckRequestMetadata {
    schemaVersion?: string;
    snapToken?: string;
    depth?: number;
  }
  
  export interface Entity {
    type?: string;
    id?: string;
  }
  
  export interface Subject {
    type?: string;
    id?: string;
    relation?: string;
  }
  
  export interface ContextAttribute {
    name?: string;
  }
  
  export interface ComputedAttribute {
    name?: string;
  }
  
  export interface Argument {
    computedAttribute?: ComputedAttribute;
    contextAttribute?: ContextAttribute;
  }
  
  export interface Context {
    tuples?: Array<{
      entity?: Entity;
      relation?: string;
      subject?: Subject;
    }>;
    attributes?: Array<{
      entity?: Entity;
      attribute?: string;
      value?: any;
    }>;
    data?: Record<string, any>;
  }
  
  export interface CheckBody {
    metadata?: PermissionCheckRequestMetadata;
    entity?: Entity;
    permission?: string;
    subject?: Subject;
    context?: Context;
    _arguments?: Argument[];
  }
  
  export interface PermissionCheckResponse {
    // Add the response type based on your API requirements
    allowed: boolean;
    [key: string]: any;
  }