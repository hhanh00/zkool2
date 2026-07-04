# Multi currency

Currently the app only supports USD as Fiat currency. This feature is
about adding the ability to select the fiat currency from a list.

## Supported currencies
- List is obtained from coingecko

## Dynamic Currency
- Change wherever the app hardcodes USD and replace with the currency
stored in the AppSettings
- The currency is stored in the DB as a property (not in SharedPreferences)
as it affects the price column of transactions and the budget
- Retrieving the fx rate returns a JSON that has a field set as the currency.
It should be parsed dynamically
- The exchange rate should be represented as a discreet button (no shadow,
no elevation, outline).
    - Tapping on it, refreshes the rate
    - It should display the exchange rate like: "1 ZEC = 445 USD". But if the
    value would be less than 1, it should be scaled up. Ex: "100 ZEC = 1 BTC"
    - The button should be on the Account List and also added to the Account Page
    below the account value in fiat

## Changing currency
- On the Settings Page, the current currency is displayed. Tapping on it navigates
to the "Change Currency" page.
    - The page displays an alphabetical list of all supported currencies
    - Current currency is highlighted
    - Tapping on a row selects that currency, moves the highlight but does not
    change the currency in the db
    - When the user pops the page:
        - If the currency has not changed, pop
        - If the currency changed, show a confirmation dialog box that
            also asks for the exchange rate between the old currency and the new one
            explaining that the app needs to update the historical prices
            - prefill with the current exchange rate, or blank if not available (async)
            - the value is required, must be > 0
            - if the user cancels, the page is not popped and the previous currency is
            restored
            - if the user oks, call the rust fn "update_historical_prices" passing
            the exchange rate
    - update_historical_prices should recompute the value of every price column
        then update the currency property (in a single database transaction)



