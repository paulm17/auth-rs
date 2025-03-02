export interface TenantCreateRequest {
    id?: string;
    name?: string;
  }
  
  export interface Tenant {
    id?: string;
    name?: string;
    createdAt?: Date;
  }
  
  export interface TenantCreateResponse {
    tenant?: Tenant;
  }

  export interface TenantDeleteResponse {
    tenant?: Tenant;
  }

  export interface TenantListRequest {
    pageSize?: number;
    continuousToken?: string;
  }
  
  export interface TenantListResponse {
    tenants: Tenant[];
    continuousToken?: string;
  }