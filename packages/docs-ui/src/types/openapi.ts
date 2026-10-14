export namespace OpenAPIV3 {
  export type NonArraySchemaObjectType =
    | 'null'
    | 'boolean'
    | 'object'
    | 'number'
    | 'string'
    | 'integer';

  export type SchemaObject = {
    type?: string;
    description?: string;
    required?: string[];
    properties?: Record<string, any>;
    [key: string]: any;
  };
}

export type SchemaObject = OpenAPIV3.SchemaObject;
