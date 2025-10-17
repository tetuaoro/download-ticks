# Binance Klines Fetcher

A **command-line tool** to fetch and save historical candlestick (kline) data from Binance.
Supports **pagination** to handle Binance's 1000-candle limit and saves data in **JSON format**.

---

## **Example**

1. Fetch last candles

```bash
download-ticks --symbol BTCUSDT --interval H1
```

2. Fetch the candles for a range

```bash
download-ticks -s BTCUSDT -i H1 --from-date "2019-05-01T00:00:00Z" --to-date "2019-05-02T00:00:00Z"
```

3. Fetch the candles for a range and save to a json file

```bash
download-ticks -s BTCUSDT -i H1 -f "2019-05-01T00:00:00Z" -t "2019-05-02T00:00:00Z" --output-file output.json
```

## **Contributing**

Contributions are welcome! Open an issue or submit a pull request.

## License

This project is licensed under the [MIT License](./LICENSE).