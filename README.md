# yp-parser

Библиотека, поддерживающая чтение и запись txt, csv, бинарного формата, и набор CLI-утилит для работы с файлами из командной строки.

## Форматы
Точная структура форматов [`доступна тут`](https://code.s3.yandex.net/middle-rust/%D0%A1%D0%BF%D0%B5%D1%86%D0%B8%D1%84%D0%B8%D0%BA%D0%B0%D1%86%D0%B8%D1%8F_%D1%84%D0%BE%D1%80%D0%BC%D0%B0%D1%82%D0%BE%D0%B2.zip).

## Использование CLI

### `reader`

Читает файл и печатает транзакции в человекочитаемом виде.

```bash
cargo run --bin reader -- txt ./transactions.txt
```

### `converter`

Конвертирует файл из одного формата в другой.

```bash
cargo run --bin converter -- \
  --input ./transactions.txt \
  --input-format txt \
  --output-format csv > transactions.csv
```

### `comparer`

Сравнивает два файла и показывает отличия. Если одна и та же транзакция встречается разное число раз, это тоже считается отличием.

```bash
cargo run --bin comparer -- \
  --file1 ./a.txt --format1 txt \
  --file2 ./b.csv --format2 csv
```

## Использование как библиотеки

Пример чтения CSV:

```rust
use std::fs::File;
use yp_parser::{CsvParser, Parser};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open("transactions.csv")?;
    let txs = CsvParser::from_read(&mut file)?;
    println!("parsed: {}", txs.len());
    Ok(())
}
```

Пример записи бинарного файла:

```rust
use std::fs::File;
use yp_parser::{BinaryParser, Parser, Transaction};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let txs = vec![Transaction::new()];
    let mut file = File::create("transactions.bin")?;
    BinaryParser::write_to(&mut file, &txs)?;
    Ok(())
}
```
