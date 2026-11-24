fn engraving_luminance(tile: AtlasTile, portrait: vec2<f32>) -> f32 {
    let size = vec2<f32>(tile.bounds.size);
    let texel = vec2<f32>(tile.bounds.origin) + clamp(portrait * size, vec2<f32>(1.0), size - 1.0);
    let atlas_size = vec2<f32>(textureDimensions(t_sprite, 0));
    return textureSampleLevel(t_sprite, s_sprite, texel / atlas_size, 0.0).r;
}

fn engraving_rounded(point: vec2<f32>, size: vec2<f32>, radius: f32) -> f32 {
    let corner = abs(point - size * 0.5) - size * 0.5 + radius;
    return length(max(corner, vec2<f32>(0.0))) + min(max(corner.x, corner.y), 0.0) - radius;
}

fn engraving(sprite: PolychromeSprite, position: vec2<f32>) -> vec4<f32> {
    let paper = vec3<f32>(0.961, 0.957, 0.941);
    let ink = vec3<f32>(0.075, 0.074, 0.071);
    let night = vec3<f32>(0.043, 0.043, 0.047);
    let chalk = vec3<f32>(0.930, 0.925, 0.895);

    let reveal = -sprite.corner_radii.top_left - 1.0;
    let spacing = -sprite.corner_radii.top_right;
    let tilt = -sprite.corner_radii.bottom_right - 2.0;
    let horizon = -sprite.corner_radii.bottom_left - 1.0;
    let is_scene = horizon > 0.5;

    let size = sprite.bounds.size;
    let point = position - sprite.bounds.origin;
    let scene_portrait = vec2<f32>((point.x / size.x - 0.5) * 0.8 + 0.5, point.y / size.x * 0.8 + 0.1);
    let cover_portrait = (point / size - 0.5) * 0.9 + 0.5;
    let portrait = select(cover_portrait, scene_portrait, is_scene);
    let light = engraving_luminance(sprite.tile, portrait);

    let direction = normalize(vec2<f32>(tilt, 1.0));
    let wave = dot(point, direction) - light * spacing * 1.45;
    let ridge = abs(fract(wave / spacing) - 0.5) * 2.0;
    let edge = 1.45 / spacing;
    let dark_width = mix(0.05, 0.86, pow(1.0 - light, 1.15));
    let light_width = mix(0.06, 0.90, pow(light, 1.1));
    let dark_line = 1.0 - smoothstep(dark_width - edge, dark_width + edge, ridge);
    let light_line = 1.0 - smoothstep(light_width - edge, light_width + edge, ridge);

    let depth = point.y / max(horizon, 1.0);
    let scene_mask = (1.0 - smoothstep(0.6, 1.0, depth)) * smoothstep(0.097, 0.2625, point.y / size.x);
    let mask = select(1.0, scene_mask, is_scene);

    let source = vec2<f32>(size.x * 0.5, size.y * 0.939);
    let reach = distance(point, source) / size.y;
    let night_share = smoothstep(reach - 0.16, reach + 0.02, reveal * 1.36 - 0.2);

    let day_color = mix(paper, ink, dark_line * mask * 0.92);
    let night_color = mix(night, chalk, light_line * mask);
    let color = mix(day_color, night_color, night_share);

    let radius = size.x * select(0.053, 0.0455, is_scene);
    return vec4<f32>(color, saturate(0.5 - engraving_rounded(point, size, radius)));
}
