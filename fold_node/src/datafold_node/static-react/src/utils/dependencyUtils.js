export function getSchemaDependencies(schemas) {
  const deps = {}
  schemas.forEach(schema => {
    deps[schema.name] = {}
    if (!schema.fields) return
    Object.values(schema.fields).forEach(field => {
      if (field.field_mappers) {
        Object.keys(field.field_mappers).forEach(srcSchema => {
          if (srcSchema && srcSchema !== schema.name) {
            if (!deps[schema.name][srcSchema]) deps[schema.name][srcSchema] = new Set()
            deps[schema.name][srcSchema].add('field mapper')
          }
        })
      }
      if (field.transform && Array.isArray(field.transform.inputs)) {
        field.transform.inputs.forEach(input => {
          const [schemaName] = input.split('.')
          if (schemaName && schemaName !== schema.name) {
            if (!deps[schema.name][schemaName]) deps[schema.name][schemaName] = new Set()
            deps[schema.name][schemaName].add('transform')
          }
        })
      }
    })
  })
  const result = {}
  Object.entries(deps).forEach(([schemaName, map]) => {
    result[schemaName] = Object.entries(map).map(([dep, kinds]) => ({
      schema: dep,
      types: Array.from(kinds)
    }))
  })
  return result
}
