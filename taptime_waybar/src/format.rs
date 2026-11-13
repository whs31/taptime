use serde_json::Value;

pub fn duration(value: i64, seconds: bool) -> String {
  let value = value.unsigned_abs();
  let base = format!("{:02}:{:02}", value / 3600, value / 60 % 60);
  if seconds {
    format!("{base}:{:02}", value % 60)
  } else {
    base
  }
}

pub fn format(
  value: &Value,
  template: &str,
  icon: &str,
  seconds: bool,
  positive: &str,
  negative: &str,
) -> String {
  let number = |key: &str| value[key].as_i64().unwrap_or(0);
  let balance = number("balance_seconds");
  let mut fields = vec![
    ("icon", icon.to_owned()),
    ("date", value["date"].as_str().unwrap_or("").into()),
    (
      "active",
      value["active"].as_bool().unwrap_or(false).to_string(),
    ),
  ];
  for (name, key) in [
    ("worked", "worked_seconds"),
    ("presence", "presence_seconds"),
    ("work_target", "work_target_seconds"),
    ("presence_target", "presence_target_seconds"),
  ] {
    fields.push((
      name,
      if value[key].is_number() {
        duration(number(key), seconds)
      } else {
        "--:--".into()
      },
    ));
  }
  fields.push((
    "balance",
    if value["balance_seconds"].is_number() {
      (if balance >= 0 { positive } else { negative })
        .replace("{value}", &duration(balance, seconds))
    } else {
      "--:--".into()
    },
  ));
  let monthly = number("month_balance_seconds");
  fields.push((
    "month_balance",
    if value["month_balance_seconds"].is_number() {
      (if monthly >= 0 { positive } else { negative })
        .replace("{value}", &duration(monthly, seconds))
    } else {
      "--:--".into()
    },
  ));
  fields.push((
    "remaining",
    duration(
      number("presence_target_seconds")
        .saturating_sub(number("presence_seconds"))
        .max(0),
      seconds,
    ),
  ));
  fields.iter().fold(template.to_owned(), |text, (key, val)| {
    text.replace(&format!("{{{key}}}"), val)
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn lunch_target_negative_balance_and_unknown_tokens() {
    let value = serde_json::json!({"worked_seconds":22500,"presence_seconds":22500,"presence_target_seconds":30600,"balance_seconds":-8100});
    assert_eq!(
      format(
        &value,
        "{worked} {balance} {remaining} {custom}",
        "",
        false,
        "+{value}",
        "-{value}"
      ),
      "06:15 -02:15 02:15 {custom}"
    );
  }
}
