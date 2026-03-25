use pulldown_cmark::Parser;

pub fn render(message: &str) -> String {
    let parser = Parser::new_ext(message, super::markdown_options());
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Useful API</title>
    <style>
        body {{
            background-color: #222;
            color: #eee;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            margin: 0;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
            text-align: center;
            padding: 2rem;
            box-sizing: border-box;
            font-size: 1.5rem;
            line-height: 1.4;
        }}
        div.container {{
            max-width: 800px;
        }}
        a {{
            color: #8ebcf1;
            text-decoration: none;
        }}
        a:hover {{
            text-decoration: underline;
        }}
        ul {{
            text-align: left;
            display: inline-block;
            margin: 1rem 0;
        }}
        li {{
            margin: 0.5rem 0;
        }}
    </style>
</head>
<body>
    <div class="container">{}</div>
</body>
</html>"#,
        html_output
    )
}
