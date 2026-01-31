# Refund Processing System

A system for handling customer refund requests in an e-commerce platform.

## Domain Model

### Refund Request
The main entity representing a customer's refund request.

Fields:
- id: unique identifier (UUID)
- order_id: reference to the original order
- customer_id: reference to the customer
- amount: monetary value to refund
- reason: text description of refund reason
- status: current status of the refund (pending, approved, rejected, processed)
- created_at: timestamp when request was created
- processed_at: timestamp when refund was processed (optional)

### Refund Response
Response returned after processing a refund.

Fields:
- refund_id: the ID of the created/processed refund
- status: current status
- message: human-readable status message

### Order
Represents an original order that may be refunded.

Fields:
- id: unique identifier
- customer_id: customer who placed the order
- total: order total amount
- payment_id: reference to the payment transaction
- created_at: when the order was placed

## External Integrations

### Payment Gateway (Stripe)
Used to process refund transactions.

Base URL: https://api.stripe.com/v1

Operations:
- POST /refunds - Create a refund for a previous charge
  - Input: charge_id, amount, reason
  - Output: refund_id, status, amount

### Notification Service
Internal service for sending notifications.

Base URL: https://notifications.internal.example.com

Operations:
- POST /email - Send an email notification
  - Input: to, subject, body, template_id
  - Output: message_id, status

## Business Processes

### Process Refund
The main workflow for handling a refund request.

Steps:
1. Validate the refund request
   - Check amount is positive
   - Check amount doesn't exceed order total
2. Look up the original order from database
3. Verify the customer owns this order
4. Call Payment Gateway to process the refund
5. Update refund status in database
6. Send confirmation email to customer
7. Return the refund result

Error conditions:
- INVALID_AMOUNT: Amount is zero or negative
- AMOUNT_EXCEEDS_ORDER: Refund amount greater than order total
- ORDER_NOT_FOUND: Referenced order doesn't exist
- UNAUTHORIZED: Customer doesn't own this order
- PAYMENT_FAILED: Payment gateway returned an error

## API Endpoints

### POST /api/v1/refunds
Create a new refund request.

- Input: RefundRequest
- Output: RefundResponse
- Workflow: ProcessRefund
- Authentication: Required (user scope: refunds:write)
- Timeout: 5000ms
- Retries: 3 with exponential backoff

Errors:
- 400 Bad Request: INVALID_AMOUNT, AMOUNT_EXCEEDS_ORDER
- 401 Unauthorized: UNAUTHORIZED
- 404 Not Found: ORDER_NOT_FOUND
- 502 Bad Gateway: PAYMENT_FAILED

### GET /api/v1/refunds/{id}
Get refund status by ID.

- Input: refund_id (path parameter)
- Output: RefundResponse
- Authentication: Required (user scope: refunds:read)
