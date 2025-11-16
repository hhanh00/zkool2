# Auto Import Tool для Ywallet

Утилита для автоматического импорта сид-фраз и создания множественных аккаунтов в Ywallet БД.

## Описание

Эта утилита позволяет:
- Импортировать сид-фразу в базу данных Ywallet
- Автоматически создавать несколько аккаунтов с разными account index
- Генерировать все необходимые ключи (transparent, sapling, orchard)
- Работать с зашифрованными БД (sqlcipher)

## Установка

```bash
cd auto_import
cargo build --release
```

Скомпилированный бинарник будет находиться в `target/release/auto_import`

## Использование

### Базовый пример

```bash
./auto_import \
  --db-path /path/to/zkool.db \
  --seed-phrase "ваша сид фраза из 12 или 24 слов" \
  --count 5
```

Это создаст 5 аккаунтов с индексами 0, 1, 2, 3, 4 (пути m/44'/133'/0', m/44'/133'/1', и т.д.)

### Все параметры

```bash
./auto_import \
  --db-path /path/to/zkool.db \
  --seed-phrase "abandon abandon ... art" \
  --count 10 \
  --start-index 0 \
  --name-prefix "MyWallet" \
  --passphrase "optional passphrase" \
  --birth 419200 \
  --db-password "password_if_encrypted"
```

#### Параметры:

- `-d, --db-path <PATH>` - **обязательно** - путь к файлу БД Ywallet
- `-s, --seed-phrase <PHRASE>` - **обязательно** - сид-фраза (12-24 слова)
- `-c, --count <NUMBER>` - **обязательно** - количество аккаунтов для создания
- `-i, --start-index <NUMBER>` - начальный account index (по умолчанию: 0)
- `-n, --name-prefix <STRING>` - префикс имени аккаунта (по умолчанию: "Account")
- `-p, --passphrase <STRING>` - дополнительная passphrase для сид-фразы (опционально)
- `-b, --birth <HEIGHT>` - block height когда кошелек был создан (опционально, по умолчанию: Sapling activation)
- `--db-password <PASSWORD>` - пароль БД если она зашифрована (опционально)

### Примеры

#### Создать 3 аккаунта начиная с индекса 5:

```bash
./auto_import \
  --db-path ~/Documents/zkool.db \
  --seed-phrase "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about" \
  --count 3 \
  --start-index 5 \
  --name-prefix "Imported"
```

Это создаст аккаунты:
- "Imported 5" с путем m/44'/133'/5'
- "Imported 6" с путем m/44'/133'/6'
- "Imported 7" с путем m/44'/133'/7'

#### Работа с зашифрованной БД:

```bash
./auto_import \
  --db-path ~/Documents/zkool.db \
  --db-password "mypassword" \
  --seed-phrase "your seed phrase here" \
  --count 5
```

#### С passphrase (как в Trezor):

```bash
./auto_import \
  --db-path ~/Documents/zkool.db \
  --seed-phrase "your seed phrase here" \
  --passphrase "my secret passphrase" \
  --count 5
```

#### Указать birth height для быстрой синхронизации:

```bash
./auto_import \
  --db-path ~/Documents/zkool.db \
  --seed-phrase "your seed phrase here" \
  --count 5 \
  --birth 1500000
```

## Важные замечания

1. **Резервное копирование**: ОБЯЗАТЕЛЬНО создайте резервную копию БД перед использованием:
   ```bash
   cp ~/Documents/zkool.db ~/Documents/zkool.db.backup
   ```

2. **Account Index (aindex)**: Определяет путь деривации m/44'/133'/X'/0/0, где X - это aindex

3. **Birth Height**: Если вы знаете block height когда кошелек был создан, укажите его для более быстрой синхронизации

4. **Passphrase**: Это НЕ пароль БД! Это дополнительное слово для BIP39 мнемоники (как в Trezor)

5. **Pools**: Утилита создает ключи для всех пулов (transparent, sapling, orchard)

## Как это работает

Утилита повторяет ту же логику что и функция `new_account()` в основном коде Ywallet:

1. Валидирует сид-фразу
2. Подключается к БД
3. Для каждого аккаунта:
   - Создает запись в таблице `accounts`
   - Генерирует UnifiedSpendingKey из seed + aindex
   - Деривирует ключи для transparent, sapling, orchard пулов
   - Сохраняет все ключи и адреса в соответствующие таблицы БД
   - Инициализирует sync heights

## Проверка результата

После запуска утилиты:
1. Откройте Ywallet
2. Выберите БД
3. Вы должны увидеть все созданные аккаунты

## Troubleshooting

### "Invalid seed phrase"
- Проверьте что сид-фраза правильная (12, 18, 21 или 24 слова)
- Убедитесь что слова разделены пробелами

### "Failed to connect to database"
- Проверьте путь к БД
- Если БД зашифрована, укажите пароль через `--db-password`

### "Account creation failed"
- Возможно аккаунт с таким индексом уже существует
- Попробуйте использовать другой `--start-index`

## Безопасность

⚠️ **ВНИМАНИЕ**: Эта утилита работает напрямую с БД и приватными ключами.

- Никогда не передавайте сид-фразу через незащищенные каналы
- Используйте только на доверенных машинах
- Удаляйте историю команд после использования
- Создавайте резервные копии БД

## Лицензия

Использование на свой риск. Всегда делайте резервные копии!
