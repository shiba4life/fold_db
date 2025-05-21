import assert from 'node:assert'
import { test } from 'node:test'
import { getSchemaDependencies } from '../src/utils/dependencyUtils.js'

test('getSchemaDependencies computes dependencies', () => {
  const schemas = [
    { name: 'A', fields: { a1: { field_mappers: {}, transform: null } } },
    { name: 'B', fields: { b1: { field_mappers: { A: 'a1' } } } },
    { name: 'C', fields: { c1: { transform: { inputs: ['B.b1', 'A.a1'] } } } }
  ]
  const deps = getSchemaDependencies(schemas)
  assert.deepStrictEqual(deps.A, [])
  assert.deepStrictEqual(deps.B, [
    { schema: 'A', types: ['field mapper'] }
  ])
  assert.deepStrictEqual(deps.C, [
    { schema: 'B', types: ['transform'] },
    { schema: 'A', types: ['transform'] }
  ])
})
