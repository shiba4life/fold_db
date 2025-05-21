export function getSchemaDependencies(schemas) {
  const deps = {}
  schemas.forEach(schema => {
    deps[schema.name] = new Map()
  })

  schemas.forEach(schema => {
    if (!schema.fields) return
    Object.values(schema.fields).forEach(field => {
      if (field.field_mappers) {
        Object.keys(field.field_mappers).forEach(srcSchema => {
          if (srcSchema && srcSchema !== schema.name) {
            if (!deps[schema.name].has(srcSchema)) {
              deps[schema.name].set(srcSchema, new Set())
            }
            deps[schema.name].get(srcSchema).add('field_mapper')
          }
        })
      }
      if (field.transform && Array.isArray(field.transform.inputs)) {
        field.transform.inputs.forEach(input => {
          const [schemaName] = input.split('.')
          if (schemaName && schemaName !== schema.name) {
            if (!deps[schema.name].has(schemaName)) {
              deps[schema.name].set(schemaName, new Set())
            }
            deps[schema.name].get(schemaName).add('transform')
          }
        })
      }
    })
  })

  return Object.fromEntries(
    Object.entries(deps).map(([schema, map]) => [
      schema,
      Array.from(map.entries()).map(([depSchema, types]) => ({
        schema: depSchema,
        types: Array.from(types)
      }))
    ])
  )
}

export function getDependencyGraph(schemas) {
  const deps = getSchemaDependencies(schemas)
  const nodes = schemas.map(s => s.name)
  const edges = []

  Object.entries(deps).forEach(([target, arr]) => {
    arr.forEach(dep => {
      dep.types.forEach(type => {
        edges.push({ source: dep.schema, target, type })
      })
    })
  })

  return { nodes, edges }
}
