use serde_json::{Value, json};

use crate::config::Config;

pub fn render(value: &Value, config: &Config, disconnected: bool) -> Value {
  let balance = value["balance_seconds"].as_i64().unwrap_or(0);
  let expand = |template: &str| {
    let mut template = template.to_owned();
    for (name, key) in [
      ("worked", "worked_seconds"),
      ("presence", "presence_seconds"),
      ("work_target", "work_target_seconds"),
      ("presence_target", "presence_target_seconds"),
    ] {
      if let Some(seconds) = value[key].as_i64() {
        let n = seconds.unsigned_abs();
        let mut parts = Vec::new();
        if config.hours {
          parts.push(format!("{:02}", n / 3600));
        }
        if config.minutes {
          parts.push(format!(
            "{:02}",
            if config.hours { n / 60 % 60 } else { n / 60 }
          ));
        }
        if config.seconds {
          parts.push(format!(
            "{:02}",
            if config.minutes || config.hours {
              n % 60
            } else {
              n
            }
          ));
        }
        template = template.replace(&format!("{{{name}}}"), &parts.join(":"));
      }
    }
    template = template.replace(
      "{active}",
      if value["active"] == true {
        &config.active_text
      } else {
        &config.inactive_text
      },
    );
    if !config.show_balance {
      template = template.replace("{balance}", "");
    }
    taptime_waybar::format::format(
      value,
      &template,
      &config.icon,
      config.seconds,
      &config.positive,
      &config.negative,
    )
  };
  let stale = disconnected || value["stale"].as_bool().unwrap_or(true);
  let mut classes = vec!["taptime"];
  if stale {
    classes.push(&config.stale_class);
  }
  if disconnected {
    classes.push(&config.disconnected_class);
  }
  if balance > 0 {
    classes.push(&config.positive_class);
  }
  if !value["active"].as_bool().unwrap_or(false) {
    classes.push(&config.inactive_class);
  }
  let text = if disconnected && config.unavailable == "hide" {
    String::new()
  } else if disconnected && config.unavailable == "indicator" {
    config.unavailable_text.clone()
  } else {
    expand(&config.format)
  };
  json!({
    "worked": expand("{worked}"),
    "balance": expand("{balance}"),
    "balance_positive": balance > 0,
    "month_balance_positive": value["month_balance_seconds"].as_i64().is_some_and(|seconds| seconds > 0),
    "start_time": value["first_check_in"],
    "end_time": value["last_check_out"],
    "month_balance": taptime_waybar::format::format(
      value, "{month_balance}", &config.icon,
      config.seconds, &config.positive, &config.negative
    ),
    "active": value["active"],
    "date": value["date"],
    "text": text, "tooltip": format!("{}{}", expand(&config.tooltip), if disconnected { &config.unavailable_text } else { "" }), "class": classes, "alt": if stale { "stale" } else { "ready" }})
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn signed_balance_and_unknown_variables() {
    let config = Config {
      format: "{worked} {balance} {unknown}".into(),
      ..Config::default()
    };
    let value =
      json!({"worked_seconds": 32580, "balance_seconds": 1980, "active": true, "stale": false});
    assert_eq!(
      render(&value, &config, false)["text"],
      "09:03 +00:33 {unknown}"
    );
    assert_eq!(taptime_waybar::format::duration(-8100, false), "02:15");
    assert_eq!(taptime_waybar::format::duration(3661, true), "01:01:01");
    assert_eq!(render(&value, &config, false)["worked"], "09:03");
    assert_eq!(render(&value, &config, false)["balance"], "+00:33");
    assert_eq!(render(&value, &config, false)["balance_positive"], true);
    let mut with_month = value.clone();
    with_month["month_balance_seconds"] = json!(2340);
    assert_eq!(
      render(&with_month, &config, false)["month_balance"],
      "+00:39"
    );
    assert_eq!(
      render(&with_month, &config, false)["month_balance_positive"],
      true
    );
    assert_eq!(render(&value, &config, true)["alt"], "stale");
  }
}
