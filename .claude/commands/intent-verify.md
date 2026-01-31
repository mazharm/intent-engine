# Verify All Intents

Run complete verification pipeline (formatting, validation, generation check).

**Usage:** `/intent-verify`

## Verification Steps

1. **Format Check** - Verify all intent files are properly formatted
2. **Validation** - Run full semantic validation
3. **Generation Check** - Verify generated code matches intents
4. **Obligations Check** - Check for open high-severity obligations

## Process

### Step 1: Format Check
- Read all intent files
- Check if they match canonical JSON format
- Fail if any file needs formatting

### Step 2: Validation
- Run all validation checks (see `/intent-validate`)
- Fail if any errors found

### Step 3: Generation Check
- Compare current `gen/` directory with what would be generated
- Fail if any drift detected

### Step 4: Obligations Check
- Check for ContractTest and Migration intents
- Identify any with HIGH severity that are not fulfilled
- Fail if open obligations exist

## Output Format

### Success
```
Verification passed.
  15 intents validated
  12 files generated
  3 obligations (0 open)
```

### Failure Examples

Format failure:
```
Verification failed: 2 files need formatting
  .intent/model/User.intent.json
  .intent/model/Order.intent.json

Run /intent-fmt to fix.
```

Validation failure:
```
Verification failed: 3 validation errors

  [E001] Unknown type reference: InvalidType
  [E002] Missing workflow: OrderWorkflow
  [E003] Circular dependency detected

Run /intent-validate for details.
```

Generation drift:
```
Verification failed: Generated code does not match intents

  gen/src/types.rs - outdated
  gen/src/endpoints/create_user.rs - missing

Run /intent-gen to regenerate.
```

Open obligations:
```
Verification failed: 1 open HIGH severity obligation

  [HIGH] UserApiContract - Missing test for DeleteUser endpoint

Address the obligation or reduce its severity.
```

## CI/CD Usage

This command is designed for CI/CD pipelines. It returns success only if all checks pass, making it suitable for:

- Pre-commit hooks
- Pull request checks
- Deployment gates

## Exit Behavior

Reports first failure and stops. Fix issues in order:
1. Format issues → run `/intent-fmt`
2. Validation errors → fix intent files
3. Generation drift → run `/intent-gen`
4. Open obligations → address or acknowledge
