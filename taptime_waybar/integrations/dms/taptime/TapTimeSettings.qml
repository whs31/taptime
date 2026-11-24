import QtQuick
import qs.Common
import qs.Widgets
import qs.Modules.Plugins

PluginSettings {
    id: root
    pluginId: "taptime"

    SelectionSetting {
        settingKey: "pillText"
        label: I18n.trFor("taptime", "Bar text")
        description: I18n.trFor("taptime", "What the bar pill shows next to the progress ring")
        options: [
            {
                label: I18n.trFor("taptime", "Worked time"),
                value: "worked"
            },
            {
                label: I18n.trFor("taptime", "Time left"),
                value: "remaining"
            },
            {
                label: I18n.trFor("taptime", "Leave-at time"),
                value: "leaveAt"
            },
            {
                label: I18n.trFor("taptime", "Ring only"),
                value: "none"
            }
        ]
        defaultValue: "worked"
    }

    ToggleSetting {
        settingKey: "showBalance"
        label: I18n.trFor("taptime", "Show day balance")
        description: I18n.trFor("taptime", "Highlighted once it turns positive")
        defaultValue: true
    }

    ToggleSetting {
        settingKey: "showRing"
        label: I18n.trFor("taptime", "Progress ring")
        description: I18n.trFor("taptime", "Ring fills up towards the daily target; the dot is filled while checked in")
        defaultValue: true
    }

    SelectionSetting {
        settingKey: "clickAction"
        label: I18n.trFor("taptime", "Left click")
        description: I18n.trFor("taptime", "Right click does the other one")
        options: [
            {
                label: I18n.trFor("taptime", "Open details"),
                value: "popout"
            },
            {
                label: I18n.trFor("taptime", "Open dashboard"),
                value: "dashboard"
            }
        ]
        defaultValue: "popout"
    }

    StyledText {
        width: parent.width
        wrapMode: Text.WordWrap
        font.pixelSize: Theme.fontSizeSmall
        color: Theme.surfaceVariantText
        text: I18n.trFor("taptime", "Time format, dashboard URL and notifications are configured in ~/.config/taptime-waybar/config.toml. Toggle the details with: dms ipc call widget toggle taptime")
    }
}
