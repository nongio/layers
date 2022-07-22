# Animation
- start
- duration
- easing

# Timeline
                            T
prop1              ◆        |     ◆
prop2                 ◆     |        ◆
prop3                       | ◆                   ◆
prop4                       | ◆              ◆
          

Animations
[1](animation))
[2](animation))0.3
[3](animation))0.4
[4](animation))0.8


(Model, Dirty, Render)



Prop{
    value: T
}

Animation {
    from: T,
    to: T,
    start: f64,
    duration: f64,
    iteration_count: u32,
    direction: AnimationDirection,
    easing: Easing,
    speed: f64,
}
enum AnimationDirection {
    Normal,
    Reverse,
    Alternate,
    AlternateReverse,
}

Entity (id, Model, NeedsRender, Render)

Model {
    id,
    props: IndexMap<id, AnimatedValue>,
}

Change {
    entity_id,
    (id, property),
    animation,
}

Systems:
- update animations
- execute changes
    prop = lerp(from, to, t)
- render enetities
    if needs_render
        render_entities

async
- clean finished animations
- clean changes




change = entity.property.animate(new_value)

Entity {
    id,
    properties: HashMap<String, Prop>,
}

