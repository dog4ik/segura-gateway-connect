## Gateway.Connect SeguraPay integration

This project integrates SeguraPay gateway with reactivepay platform using Gateway.Connect API

### Settings example

```
{
  "USD": {
    "gateways": {
      "pay": {
        "default": "segura"
      }
    }
  },
  "gateways": {
    "allow_h2h_payin_without_card": true,
    "allow_host2host": true,
    "segura": {
      "class": "segura",
      "client_id": "TSPIOAJP-976011-20251016",
      "secret": "TSaAbqlSeUhBBC8Kfrf4UOb1kgwCDtFPRh4LdFiWexVneL4Vz9Ng81953",
      "sign_key": "e7403b3c0d76a35312e7cc65eeb75808"
    }
  }
}
```

### RP request minimal request body:

H2H
```
{
  "product": "Your Product",
  "amount": 100,
  "currency": "USD",
  "orderNumber": "09873f72-d21d-43e7-9c07-31eb74b3d55e",
  "callbackUrl": "https://example.com",
  "card": {
    "holder": "Test testov",
    "cvv": "123",
    "pan": "4485081333091151",
    "expires": "11/2029"
  },
  "customer": {
    "email": "test@gmail.com",
    "address": "Sin city 3",
    "city": "Leicester",
    "country": "NG",
    "last_name": "Warner",
    "first_name": "Bros",
    "phone": "4407517049134",
    "postcode": "98102",
    "state": "WA"
  }
}
```

Redirect
```
{
  "product": "Your Product",
  "amount": 100,
  "currency": "USD",
  "orderNumber": "09873f72-d21d-43e7-9c07-31eb74b3d55e",
  "callbackUrl": "https://example.com",
  "customer": {
    "email": "test@gmail.com",
    "address": "Sin city 3",
    "city": "Leicester",
    "country": "NG",
    "last_name": "Warner",
    "first_name": "Bros",
    "phone": "4407517049134",
    "postcode": "98102",
    "state": "WA"
  }
}
```

### Compile time env variables

- `DATABASE_URL` - Connection string for sqlite database
- `RP_CALLBACK_URL` - Gateway.Connect callback url override (optional)

### Runtime env variables:

- `CALLBACK_URL` - Callback url gateway should use. Should match url of the server application runs on.
- `SIGN_KEY` - Key to sign callbacks
- `PORT` - Port server runs on

### Build instructions

1. Create `database.sqlite` and execute `init.sql`
2. Set `DATABASE_URL` env variable to `sqlite://database.sqlite`
3. Run `cargo build --release`
