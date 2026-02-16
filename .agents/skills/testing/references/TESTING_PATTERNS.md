---
description: |
  Detailed testing patterns and anti-patterns.
  Includes TDD workflow, mocking strategies, and common pitfalls.
---

# Testing Patterns

## TDD Workflow

1. **Write failing test first**
2. **Implement minimum code to pass**
3. **Run tests - verify pass**
4. **Refactor if needed**
5. **Run tests again**
6. **Commit**

## Mocking Strategies

### Python (pytest-mock)

```python
def test_user_service_sends_email(mocker):
    mock_email = mocker.patch('app.services.email.send')
    create_user("test@example.com")
    mock_email.assert_called_once()
```

### JavaScript (Vitest)

```typescript
vi.mock("./api", () => ({
  fetchUser: vi.fn(),
}));
```

### Go

```go
func TestService(t *testing.T) {
    mockRepo := &MockRepository{}
    svc := NewService(mockRepo)
    // test with mock
}
```

## Anti-Patterns to Avoid

- **No throwaway tests** - All tests should be permanent
- **Testing implementation details** - Test behavior, not internals
- **No assertions** - Always assert expected outcomes
- **Flaky tests** - No timeouts or random data
- **Testing multiple things** - One assertion per test preferred

## Fixtures

### Python

```python
@pytest.fixture
def user():
    return User(name="Test", email="test@example.com")
```

### JavaScript

```typescript
const user = { name: "Test", email: "test@example.com" };
```

## Integration vs Unit Tests

- **Unit**: Fast, isolated, mock dependencies
- **Integration**: Slower, real dependencies, test boundaries
- **E2E**: Full system, real browser, slowest

Use pyramid: many unit, fewer integration, few E2E.
