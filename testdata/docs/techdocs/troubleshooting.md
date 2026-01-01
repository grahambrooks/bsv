# Troubleshooting

## Common Issues

### Payment Stuck in Pending

**Symptoms:**
- Payment status remains "pending" for more than 5 minutes
- No webhook received

**Causes:**
1. Provider timeout
2. Network connectivity issues
3. Invalid provider credentials

**Resolution:**

```bash
# Check payment status directly
curl -X GET https://api.example.com/payments/{id}/status

# Force status refresh
curl -X POST https://api.example.com/payments/{id}/refresh
```

### Duplicate Payments

**Symptoms:**
- Customer charged multiple times
- Multiple payment records for same order

**Causes:**
- Missing idempotency key
- Client retry without proper handling

**Resolution:**

Always include an idempotency key:

```bash
curl -X POST https://api.example.com/payments \
  -H "Idempotency-Key: order_123_attempt_1" \
  -d '{"amount": 2500, "currency": "USD"}'
```

### Webhook Failures

**Symptoms:**
- Events not received
- Signature validation errors

**Causes:**
- Incorrect webhook secret
- Firewall blocking requests
- Endpoint returning non-2xx

**Resolution:**

1. Verify webhook secret in dashboard
2. Check firewall rules for Stripe/PayPal IPs
3. Ensure endpoint returns 200 within 30 seconds

## Logging

Enable debug logging:

```bash
export LOG_LEVEL=debug
npm run dev
```

## Support

If issues persist:

1. Check [status page](https://status.example.com)
2. Search [known issues](https://github.com/example/payment-service/issues)
3. Contact #platform-payments on Slack
