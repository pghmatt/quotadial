
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'qd' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'qd'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'qd' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show current usage for every account this machine is signed in to')
            [CompletionResult]::new('watch', 'watch', [CompletionResultType]::ParameterValue, 'Keep showing usage, refreshing every minute. No provider is asked more often than its own limit, so most refreshes show cached numbers with their age')
            [CompletionResult]::new('daemon', 'daemon', [CompletionResultType]::ParameterValue, 'Run the daemon until Ctrl-C: the one poller every consumer shares, with history and the local API. While it runs, `status` and `watch` ask it instead of polling')
            [CompletionResult]::new('mcp', 'mcp', [CompletionResultType]::ParameterValue, 'Run the MCP server named quotadial on standard input and output, for agents such as Claude Code and Codex. Like `status`, it asks the daemon while one runs')
            [CompletionResult]::new('alerts', 'alerts', [CompletionResultType]::ParameterValue, 'List the alert rules and what they told lately, or add, remove, or turn one on or off. Rules are written in plain words, such as `quotadial alerts add "warn me at 80%"`')
            [CompletionResult]::new('doctor', 'doctor', [CompletionResultType]::ParameterValue, 'Print what support asks for, with nothing secret in it: the version, the system, the licence, the daemon, every account and where its sign-in is, the problems and the settings. The app''s Send feedback attaches the same')
            [CompletionResult]::new('licence', 'licence', [CompletionResultType]::ParameterValue, 'Say whether this copy is QuotaDial Free or QuotaDial Pro, or add or remove its licence key. A key is checked on this machine, with no network call')
            [CompletionResult]::new('history', 'history', [CompletionResultType]::ParameterValue, 'Show how each window was used over time, from the daemon''s history')
            [CompletionResult]::new('antigravity', 'antigravity', [CompletionResultType]::ParameterValue, 'Commands the Antigravity provider adds')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'qd;status' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Print JSON instead of text')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'qd;watch' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Print one line of JSON per refresh instead of text')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'qd;daemon' {
            [CompletionResult]::new('--port', '--port', [CompletionResultType]::ParameterName, 'The port for the local API, on 127.0.0.1. 0 lets the system choose a free one')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'qd;mcp' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'qd;alerts' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List the rules and what they told in the last week. The default')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a rule written in plain words, such as "warn me at 80%", "warn me when the weekly window will run out before it resets" or "tell me 30 minutes before the session resets if it is above 50%"')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove the rule with this number')
            [CompletionResult]::new('on', 'on', [CompletionResultType]::ParameterValue, 'Turn the rule with this number on')
            [CompletionResult]::new('off', 'off', [CompletionResultType]::ParameterValue, 'Turn the rule with this number off')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'qd;alerts;list' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Print JSON instead of text')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'qd;alerts;add' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'qd;alerts;remove' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'qd;alerts;on' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'qd;alerts;off' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'qd;alerts;help' {
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List the rules and what they told in the last week. The default')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a rule written in plain words, such as "warn me at 80%", "warn me when the weekly window will run out before it resets" or "tell me 30 minutes before the session resets if it is above 50%"')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove the rule with this number')
            [CompletionResult]::new('on', 'on', [CompletionResultType]::ParameterValue, 'Turn the rule with this number on')
            [CompletionResult]::new('off', 'off', [CompletionResultType]::ParameterValue, 'Turn the rule with this number off')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'qd;alerts;help;list' {
            break
        }
        'qd;alerts;help;add' {
            break
        }
        'qd;alerts;help;remove' {
            break
        }
        'qd;alerts;help;on' {
            break
        }
        'qd;alerts;help;off' {
            break
        }
        'qd;alerts;help;help' {
            break
        }
        'qd;doctor' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'qd;licence' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('show', 'show', [CompletionResultType]::ParameterValue, 'Say which QuotaDial this copy is, and from which licence. The default')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a licence key for QuotaDial Pro, which replaces any kept before')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove the licence key, which makes this copy QuotaDial Free')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'qd;licence;show' {
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Print JSON instead of text')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'qd;licence;add' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'qd;licence;remove' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'qd;licence;help' {
            [CompletionResult]::new('show', 'show', [CompletionResultType]::ParameterValue, 'Say which QuotaDial this copy is, and from which licence. The default')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a licence key for QuotaDial Pro, which replaces any kept before')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove the licence key, which makes this copy QuotaDial Free')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'qd;licence;help;show' {
            break
        }
        'qd;licence;help;add' {
            break
        }
        'qd;licence;help;remove' {
            break
        }
        'qd;licence;help;help' {
            break
        }
        'qd;history' {
            [CompletionResult]::new('--since', '--since', [CompletionResultType]::ParameterName, 'From how long ago: such as 24h, 7d or 30d, or a time such as 2099-01-01T00:00:00Z')
            [CompletionResult]::new('--provider', '--provider', [CompletionResultType]::ParameterName, 'Only this provider: claude, codex, gemini or antigravity, or copilot in a build with it')
            [CompletionResult]::new('--account', '--account', [CompletionResultType]::ParameterName, 'Only this account, by its email address or id')
            [CompletionResult]::new('--window', '--window', [CompletionResultType]::ParameterName, 'Only windows with this label, such as Session or Weekly')
            [CompletionResult]::new('--csv', '--csv', [CompletionResultType]::ParameterName, 'Print every reading as CSV, for a spreadsheet')
            [CompletionResult]::new('--json', '--json', [CompletionResultType]::ParameterName, 'Print JSON, the same as `GET /v1/history`')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'qd;antigravity' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('status-line', 'status-line', [CompletionResultType]::ParameterValue, 'Keeps the numbers Antigravity CLI''s status line hands it on standard input, then runs the status line command named after it, if any, with the same input.')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'qd;antigravity;status-line' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'qd;antigravity;help' {
            [CompletionResult]::new('status-line', 'status-line', [CompletionResultType]::ParameterValue, 'Keeps the numbers Antigravity CLI''s status line hands it on standard input, then runs the status line command named after it, if any, with the same input.')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'qd;antigravity;help;status-line' {
            break
        }
        'qd;antigravity;help;help' {
            break
        }
        'qd;help' {
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Show current usage for every account this machine is signed in to')
            [CompletionResult]::new('watch', 'watch', [CompletionResultType]::ParameterValue, 'Keep showing usage, refreshing every minute. No provider is asked more often than its own limit, so most refreshes show cached numbers with their age')
            [CompletionResult]::new('daemon', 'daemon', [CompletionResultType]::ParameterValue, 'Run the daemon until Ctrl-C: the one poller every consumer shares, with history and the local API. While it runs, `status` and `watch` ask it instead of polling')
            [CompletionResult]::new('mcp', 'mcp', [CompletionResultType]::ParameterValue, 'Run the MCP server named quotadial on standard input and output, for agents such as Claude Code and Codex. Like `status`, it asks the daemon while one runs')
            [CompletionResult]::new('alerts', 'alerts', [CompletionResultType]::ParameterValue, 'List the alert rules and what they told lately, or add, remove, or turn one on or off. Rules are written in plain words, such as `quotadial alerts add "warn me at 80%"`')
            [CompletionResult]::new('doctor', 'doctor', [CompletionResultType]::ParameterValue, 'Print what support asks for, with nothing secret in it: the version, the system, the licence, the daemon, every account and where its sign-in is, the problems and the settings. The app''s Send feedback attaches the same')
            [CompletionResult]::new('licence', 'licence', [CompletionResultType]::ParameterValue, 'Say whether this copy is QuotaDial Free or QuotaDial Pro, or add or remove its licence key. A key is checked on this machine, with no network call')
            [CompletionResult]::new('history', 'history', [CompletionResultType]::ParameterValue, 'Show how each window was used over time, from the daemon''s history')
            [CompletionResult]::new('antigravity', 'antigravity', [CompletionResultType]::ParameterValue, 'Commands the Antigravity provider adds')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'qd;help;status' {
            break
        }
        'qd;help;watch' {
            break
        }
        'qd;help;daemon' {
            break
        }
        'qd;help;mcp' {
            break
        }
        'qd;help;alerts' {
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List the rules and what they told in the last week. The default')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a rule written in plain words, such as "warn me at 80%", "warn me when the weekly window will run out before it resets" or "tell me 30 minutes before the session resets if it is above 50%"')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove the rule with this number')
            [CompletionResult]::new('on', 'on', [CompletionResultType]::ParameterValue, 'Turn the rule with this number on')
            [CompletionResult]::new('off', 'off', [CompletionResultType]::ParameterValue, 'Turn the rule with this number off')
            break
        }
        'qd;help;alerts;list' {
            break
        }
        'qd;help;alerts;add' {
            break
        }
        'qd;help;alerts;remove' {
            break
        }
        'qd;help;alerts;on' {
            break
        }
        'qd;help;alerts;off' {
            break
        }
        'qd;help;doctor' {
            break
        }
        'qd;help;licence' {
            [CompletionResult]::new('show', 'show', [CompletionResultType]::ParameterValue, 'Say which QuotaDial this copy is, and from which licence. The default')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a licence key for QuotaDial Pro, which replaces any kept before')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove the licence key, which makes this copy QuotaDial Free')
            break
        }
        'qd;help;licence;show' {
            break
        }
        'qd;help;licence;add' {
            break
        }
        'qd;help;licence;remove' {
            break
        }
        'qd;help;history' {
            break
        }
        'qd;help;antigravity' {
            [CompletionResult]::new('status-line', 'status-line', [CompletionResultType]::ParameterValue, 'Keeps the numbers Antigravity CLI''s status line hands it on standard input, then runs the status line command named after it, if any, with the same input.')
            break
        }
        'qd;help;antigravity;status-line' {
            break
        }
        'qd;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
