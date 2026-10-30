use crate::model::{DisplayMode, VirtualConfig};
use roxmltree::{Document, Node};

fn child<'a, 'input>(parent: Node<'a, 'input>, name: &str) -> Result<Node<'a, 'input>, String> {
    let mut nodes = parent.children().filter(|node| node.has_tag_name(name));
    let node = nodes
        .next()
        .ok_or_else(|| format!("驱动配置缺少 {name} 节点。"))?;
    if nodes.next().is_some() {
        return Err(format!("驱动配置存在重复的 {name} 节点，未修改。"));
    }
    Ok(node)
}

fn number(node: Node<'_, '_>) -> Result<u32, String> {
    node.text()
        .unwrap_or("")
        .trim()
        .parse()
        .map_err(|_| format!("驱动配置中的 {} 不是有效数字。", node.tag_name().name()))
}

pub fn parse(xml: &str) -> Result<VirtualConfig, String> {
    let document = Document::parse(xml).map_err(|error| format!("无法解析驱动配置：{error}"))?;
    let root = document.root_element();
    if !root.has_tag_name("vdd_settings") {
        return Err("不是受支持的 MTT 驱动配置文件。".into());
    }
    let count = number(child(child(root, "monitors")?, "count")?)?;
    let resolutions = child(root, "resolutions")?;
    let mut modes = Vec::new();
    for resolution in resolutions
        .children()
        .filter(|node| node.has_tag_name("resolution"))
    {
        let width = number(child(resolution, "width")?)?;
        let height = number(child(resolution, "height")?)?;
        for rate in resolution
            .children()
            .filter(|node| node.has_tag_name("refresh_rate"))
        {
            let mode = DisplayMode {
                width,
                height,
                refresh_hz: number(rate)?,
            };
            if !modes.contains(&mode) {
                modes.push(mode);
            }
        }
    }
    if modes.is_empty() {
        return Err("驱动配置没有有效画面模式。".into());
    }
    Ok(VirtualConfig { count, modes })
}

pub fn replace(xml: &str, config: &VirtualConfig) -> Result<String, String> {
    config.validate()?;
    // Verify the original schema before using source ranges. Unrelated XML stays byte-for-byte intact.
    parse(xml)?;
    let document = Document::parse(xml).map_err(|error| error.to_string())?;
    let root = document.root_element();
    let count = child(child(root, "monitors")?, "count")?;
    let resolutions = child(root, "resolutions")?;
    let mut groups: Vec<(u32, u32, Vec<u32>)> = Vec::new();
    for mode in &config.modes {
        if let Some(group) = groups
            .iter_mut()
            .find(|(width, height, _)| *width == mode.width && *height == mode.height)
        {
            group.2.push(mode.refresh_hz);
        } else {
            groups.push((mode.width, mode.height, vec![mode.refresh_hz]));
        }
    }
    let mut modes = String::from("<resolutions>\n");
    for (width, height, rates) in groups {
        modes.push_str(&format!(
            "    <resolution><width>{width}</width><height>{height}</height>"
        ));
        for rate in rates {
            modes.push_str(&format!("<refresh_rate>{rate}</refresh_rate>"));
        }
        modes.push_str("</resolution>\n");
    }
    modes.push_str("  </resolutions>");
    let mut replacements = [
        (count.range(), format!("<count>{}</count>", config.count)),
        (resolutions.range(), modes),
    ];
    replacements.sort_by_key(|(range, _)| std::cmp::Reverse(range.start));
    let mut updated = xml.to_owned();
    for (range, text) in replacements {
        updated.replace_range(range, &text);
    }
    parse(&updated)?;
    Ok(updated)
}
