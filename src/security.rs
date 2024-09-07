use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    pub static ref BAD_MAPS: HashMap<char, &'static str> = HashMap::from([
        ('&', "&amp;"),
        ('<', "&lt;"),
        ('>', "&gt;"),
        ('"', "&quot;"),
        ('\'', "&#x27;"),
        ('/', "&#x2F;"),
        ('`', "&grave;"),
        ('=', "&#x3D;"),
        (' ', "&nbsp;"),
    // (' ', "&#x20;"),
        ('!', "&#x21;"),
        ('"', "&#x22;"),
        ('#', "&#x23;"),
        ('$', "&#x24;"),
        ('%', "&#x25;"),
        // ('&', "&#x26;"),
    ]);
}

#[allow(dead_code)]
#[must_use]
pub fn escape_html(input: &str) -> String {
    let user_input = input.to_string();
    let mut result = String::new();

    user_input.chars().for_each(|c| {
        if let Some(replacement) = BAD_MAPS.get(&c) {
            result.push_str(replacement);
        } else {
            result.push(c);
        }
    });
    result
}

#[cfg(test)]
mod test_security {
    use super::*;

    #[test]
    fn test_escape_html() {
        let input = "<script>alert('hello')</script>";
        let expected = "&lt;script&gt;alert(&#x27;hello&#x27;)&lt;&#x2F;script&gt;";
        assert_eq!(escape_html(input), expected);
    }

    #[test]
    fn test_escape_html_with_space() {
        let input = "hello world";
        let expected = "hello&nbsp;world";
        assert_eq!(escape_html(input), expected);
    }

    #[test]
    fn test_escape_html_with_special_characters() {
        let input = "<script>alert('hello')</script> &";
        let expected = "&lt;script&gt;alert(&#x27;hello&#x27;)&lt;&#x2F;script&gt;&nbsp;&amp;";
        assert_eq!(escape_html(input), expected);
    }

    #[test]
    fn test_six_spaces() {
        let input = "      ";
        let expected = "&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;";
        assert_eq!(escape_html(input), expected);
    }

    #[test]
    fn test_escape_html_with_space_and_special_characters() {
        let input = "<script>alert('hello')</script> & world";
        let expected =
            "&lt;script&gt;alert(&#x27;hello&#x27;)&lt;&#x2F;script&gt;&nbsp;&amp;&nbsp;world";
        assert_eq!(escape_html(input), expected);
    }

    #[test]
    fn test_escape_html_with_space_and_special_characters_and_space() {
        let input = "<script>alert('hello')</script> & world ";
        let expected =
			"&lt;script&gt;alert(&#x27;hello&#x27;)&lt;&#x2F;script&gt;&nbsp;&amp;&nbsp;world&nbsp;";
        assert_eq!(escape_html(input), expected);
    }

    #[test]
    fn test_dom_xss() {
        let input =
            "<script>Docuemnt.getElementById('demo').innerHTML = 'Hello World!'</script> & world ";
        let expected = "&lt;script&gt;Docuemnt.getElementById(&#x27;demo&#x27;).innerHTML&nbsp;&#x3D;&nbsp;&#x27;Hello&nbsp;World&#x21;&#x27;&lt;&#x2F;script&gt;&nbsp;&amp;&nbsp;world&nbsp;";
        assert_eq!(escape_html(input), expected);
    }

    #[test]
    fn test_dom_xss_with_space() {
        let input =
            "<script>Docuemnt.getElementById('demo').innerHTML = 'Hello World!'</script> & world";
        let expected = "&lt;script&gt;Docuemnt.getElementById(&#x27;demo&#x27;).innerHTML&nbsp;&#x3D;&nbsp;&#x27;Hello&nbsp;World&#x21;&#x27;&lt;&#x2F;script&gt;&nbsp;&amp;&nbsp;world";
        assert_eq!(escape_html(input), expected);
    }
}
