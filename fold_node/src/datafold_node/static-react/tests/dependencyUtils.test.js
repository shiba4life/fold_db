import assert from 'node:assert'
import { test } from 'node:test'
import { getSchemaDependencies } from '../src/utils/dependencyUtils.js'

test('getSchemaDependencies computes dependencies with types', () => {
  const schemas = [
    { name: 'A', fields: { a1: { field_mappers: {}, transform: null } } },
    { name: 'B', fields: { b1: { field_mappers: { A: 'a1' } } } },
    { name: 'C', fields: { c1: { transform: { inputs: ['B.b1', 'A.a1'] } } } }
  ]
  const deps = getSchemaDependencies(schemas)
  assert.deepStrictEqual(deps.A, [])
  assert.deepStrictEqual(deps.B, [
    { schema: 'A', types: ['field_mapper'] }
  ])
  assert.deepStrictEqual(
    deps.C.sort((a, b) => a.schema.localeCompare(b.schema)),
    [
      { schema: 'A', types: ['transform'] },
      { schema: 'B', types: ['transform'] }
    ]
  )
})
