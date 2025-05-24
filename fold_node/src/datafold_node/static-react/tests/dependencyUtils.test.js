import { describe, it, expect } from 'vitest'
import { getSchemaDependencies } from '../src/utils/dependencyUtils.js'

describe('dependencyUtils', () => {
  describe('getSchemaDependencies', () => {
    it('computes dependencies with types', () => {
      const schemas = [
        { name: 'A', fields: { a1: { field_mappers: {}, transform: null } } },
        { name: 'B', fields: { b1: { field_mappers: { A: 'a1' } } } },
        { name: 'C', fields: { c1: { transform: { inputs: ['B.b1', 'A.a1'] } } } }
      ]
      const deps = getSchemaDependencies(schemas)
      
      expect(deps.A).toEqual([])
      expect(deps.B).toEqual([
        { schema: 'A', types: ['field_mapper'] }
      ])
      expect(deps.C.sort((a, b) => a.schema.localeCompare(b.schema))).toEqual([
        { schema: 'A', types: ['transform'] },
        { schema: 'B', types: ['transform'] }
      ])
    })

    it('handles empty schemas array', () => {
      const deps = getSchemaDependencies([])
      expect(deps).toEqual({})
    })

    it('handles schemas with no dependencies', () => {
      const schemas = [
        { name: 'A', fields: { a1: { field_mappers: {}, transform: null } } },
        { name: 'B', fields: { b1: { field_mappers: {}, transform: null } } }
      ]
      const deps = getSchemaDependencies(schemas)
      
      expect(deps.A).toEqual([])
      expect(deps.B).toEqual([])
    })

    it('handles complex dependency chains', () => {
      const schemas = [
        { name: 'A', fields: { a1: { field_mappers: {}, transform: null } } },
        { name: 'B', fields: { b1: { field_mappers: { A: 'a1' } } } },
        { name: 'C', fields: { c1: { field_mappers: { B: 'b1' } } } },
        { name: 'D', fields: { d1: { transform: { inputs: ['C.c1', 'A.a1'] } } } }
      ]
      const deps = getSchemaDependencies(schemas)
      
      expect(deps.A).toEqual([])
      expect(deps.B).toEqual([{ schema: 'A', types: ['field_mapper'] }])
      expect(deps.C).toEqual([{ schema: 'B', types: ['field_mapper'] }])
      expect(deps.D.sort((a, b) => a.schema.localeCompare(b.schema))).toEqual([
        { schema: 'A', types: ['transform'] },
        { schema: 'C', types: ['transform'] }
      ])
    })

    it('handles multiple dependency types for same schema', () => {
      const schemas = [
        { name: 'A', fields: { a1: { field_mappers: {}, transform: null } } },
        { name: 'B', fields: {
          b1: { field_mappers: { A: 'a1' } },
          b2: { transform: { inputs: ['A.a1'] } }
        }}
      ]
      const deps = getSchemaDependencies(schemas)
      
      expect(deps.A).toEqual([])
      expect(deps.B).toEqual([{ schema: 'A', types: ['field_mapper', 'transform'] }])
    })

    it('handles schemas with undefined or null fields', () => {
      const schemas = [
        { name: 'A', fields: { a1: { field_mappers: null, transform: undefined } } },
        { name: 'B', fields: { b1: { field_mappers: undefined, transform: null } } }
      ]
      const deps = getSchemaDependencies(schemas)
      
      expect(deps.A).toEqual([])
      expect(deps.B).toEqual([])
    })
  })
})
