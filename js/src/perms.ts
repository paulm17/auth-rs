import { BaseClient } from "./lib";
import { SchemaWriteBody, SchemaWriteResponse } from "./types/perms/schema";
import * as tenants from "./types/perms/tenant";
import { CheckBody, PermissionCheckResponse} from "./types/perms/permissions";

export class Perms extends BaseClient {
  constructor(config: { baseURL: string }) {
    super(config);
  }

  tenant = {
    create: async (body: tenants.TenantCreateRequest): Promise<tenants.TenantCreateResponse> => {
      const response = await this.fetchWithAuth('/tenants', {
        method: 'POST',
        body: JSON.stringify(body),
      });
      return response.json();
    },
    delete: async (tenantId: string): Promise<tenants.TenantDeleteResponse> => {
        const response = await this.fetchWithAuth(`/tenants/${tenantId}`, {
          method: 'DELETE'
        });
        return response.json();
      },
      list: async (options?: tenants.TenantListRequest): Promise<tenants.TenantListResponse> => {
        const queryParams = new URLSearchParams();
        if (options?.pageSize) {
          queryParams.append('page_size', options.pageSize.toString());
        }
        if (options?.continuousToken) {
          queryParams.append('continuous_token', options.continuousToken);
        }
  
        const url = `/tenants${queryParams.toString() ? `?${queryParams.toString()}` : ''}`;
        const response = await this.fetchWithAuth(url, {
          method: 'GET',
        });
        return response.json();
      }
  };

  async checkPermissions(tenantId: string, body: CheckBody): Promise<PermissionCheckResponse> {
    const response = await this.fetchWithAuth('/permissions/check', {
      method: 'POST',
      body: JSON.stringify({
        tenantId,
        ...body
      }),
    });
    return response.json();
  }

  async writeSchema(tenantId: string, body: SchemaWriteBody): Promise<SchemaWriteResponse> {
    const response = await this.fetchWithAuth(`/schemas/${tenantId}`, {
      method: 'POST', // or GET, PUT, DELETE, etc.
      body: JSON.stringify(body),
    });
    return response.json();
  }
}