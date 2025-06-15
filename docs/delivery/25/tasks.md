# Tasks for PBI 25: Unify security-related enums across modules

This document lists all tasks associated with PBI 25.

**Parent PBI**: [PBI 25: Unify security-related enums across modules](./prd.md) - **Status: InReview**

## Task Summary

| Task ID | Name | Status | Description |
| :------ | :--------------------------------------- | :------- | :--------------------------------- |
| 25-1 | [Comprehensive security enum audit and analysis](./25-1.md) | Completed | Analyze all security enums across codebase, identify duplicates and conflicts |
| 25-2 | [Create shared security types module](./25-2.md) | InProgress | Create unified security_types.rs module with all security enums |
| 25-3 | [Refactor rotation and audit modules](./25-3.md) | Completed | Update key rotation and audit modules to use shared types |
| 25-4 | [Refactor monitoring and threat detection modules](./25-4.md) | Completed | Update security monitoring and threat detection modules to use shared types |
| 25-5 | [Refactor compliance and risk management modules](./25-5.md) | Completed | Update compliance and risk management modules to use shared types |
| 25-6 | [Refactor configuration and CLI modules](./25-6.md) | Completed | Update crypto config and CLI modules to use shared types |
| 25-7 | [Update tests and fix any compilation issues](./25-7.md) | Completed | Update tests and fix any compilation issues caused by enum unification |
| 25-8 | [Update documentation and examples](./25-8.md) | Completed | Update documentation to reflect unified security types |
| 25-9 | [Final verification and cleanup](./25-9.md) | Completed | Final verification and cleanup, complete PBI 25 |