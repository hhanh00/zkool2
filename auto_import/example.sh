#!/bin/bash

# Пример использования auto_import
# ВАЖНО: Замените параметры на свои!

# Путь к скомпилированному бинарнику
BINARY="./target/release/auto_import"

# Параметры
DB_PATH="/home/kyber/Documents/zkool.db"
SEED_PHRASE="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
COUNT=5
START_INDEX=0
NAME_PREFIX="Account"
PASSPHRASE=""  # Опционально
BIRTH_HEIGHT="" # Опционально
DB_PASSWORD="" # Если БД зашифрована

# ВАЖНО: СОЗДАЙТЕ РЕЗЕРВНУЮ КОПИЮ БАЗЫ ДАННЫХ!
echo "⚠️  СОЗДАЮ РЕЗЕРВНУЮ КОПИЮ БД..."
cp "$DB_PATH" "${DB_PATH}.backup_$(date +%Y%m%d_%H%M%S)"
echo "✓ Резервная копия создана"

# Запуск утилиты
echo ""
echo "Запуск auto_import..."
echo ""

$BINARY \
  --db-path "$DB_PATH" \
  --seed-phrase "$SEED_PHRASE" \
  --count $COUNT \
  --start-index $START_INDEX \
  --name-prefix "$NAME_PREFIX"

# Если нужна passphrase:
# $BINARY \
#   --db-path "$DB_PATH" \
#   --seed-phrase "$SEED_PHRASE" \
#   --count $COUNT \
#   --passphrase "my secret passphrase"

# Если БД зашифрована:
# $BINARY \
#   --db-path "$DB_PATH" \
#   --seed-phrase "$SEED_PHRASE" \
#   --count $COUNT \
#   --db-password "database_password"

echo ""
echo "✓ Готово!"
echo ""
echo "Теперь откройте Ywallet и проверьте что аккаунты появились"
