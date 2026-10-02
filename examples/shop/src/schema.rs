pub const SCHEMA: &str = r#"
create table products (
    id          integer primary key,
    slug        text    not null unique,
    name        text    not null,
    tagline     text    not null,
    category    text    not null
        check (category in ('kitchen', 'textiles', 'tableware', 'woodwork')),
    price_cents integer not null,
    in_stock    integer not null default 1
) strict;

create table reviews (
    id         integer primary key,
    product_id integer not null references products (id) on delete cascade,
    email      text    not null,
    rating     integer not null,
    message    text    not null,
    created_at text    not null default (datetime('now'))
) strict;

create table materials (
    id   integer primary key,
    name text    not null unique,
    care text    not null default ''
) strict;

create table product_materials (
    product_id  integer not null references products (id)  on delete cascade,
    material_id integer not null references materials (id) on delete cascade,
    primary key (product_id, material_id)
) strict, without rowid;

create table cart_items (
    cart_id    text    not null,
    product_id integer not null references products (id) on delete cascade,
    qty        integer not null check (qty > 0),
    primary key (cart_id, product_id)
) strict, without rowid;

-- The first four keep ids 1-4; after them the categories take turns, so
-- shelf order is a mixed one.
insert into products (slug, name, tagline, category, price_cents, in_stock) values
    ('kettle',        'Stovetop Kettle',        'Two litres of enamelled steel, and a whistle you can hear from the next room.', 'kitchen',    4200, 1),
    ('apron',         'Canvas Apron',           'Twelve-ounce cotton, one deep pocket, straps that cross at the back.',          'textiles',   2800, 1),
    ('mug',           'Stoneware Mug',          'Three hundred and fifty millilitres under a matte glaze.',                      'tableware',  1600, 1),
    ('board',         'Walnut Board',           'Forty by twenty-eight centimetres, oiled, with a groove for the juice.',        'woodwork',   6500, 0),
    ('skillet',       'Cast Iron Skillet',      'Twenty-six centimetres, seasoned at the foundry, heavier than it looks.',       'kitchen',    5800, 1),
    ('tea-towels',    'Linen Tea Towels',       'A pair, woven tight enough to dry a glass without lint.',                       'textiles',   1800, 1),
    ('dinner-plate',  'Stoneware Dinner Plate', 'Twenty-seven centimetres with a rim that keeps the sauce in.',                  'tableware',  2200, 1),
    ('spoon',         'Olive Wood Spoon',       'One piece of wood, a long handle, no two grains alike.',                        'woodwork',   1200, 1),
    ('pepper-mill',   'Pepper Mill',            'Beech body, steel burrs, and a grind you set from the top.',                    'kitchen',    3400, 1),
    ('oven-mitt',     'Quilted Oven Mitt',      'Canvas outside, felt inside, long enough to cover the wrist.',                  'textiles',   1400, 1),
    ('tumbler',       'Glass Tumbler',          'Three hundred millilitres of plain pressed glass that stacks.',                 'tableware',   900, 1),
    ('salad-servers', 'Cherry Salad Servers',   'A fork and a spoon that darken a little every summer.',                         'woodwork',   2900, 1),
    ('colander',      'Enamel Colander',        'Three litres, two handles, feet that keep it off the sink.',                    'kitchen',    2600, 0),
    ('tablecloth',    'Hemp Tablecloth',        'Two metres forty of heavy weave that softens with every wash.',                 'textiles',   7200, 0),
    ('carafe',        'Glass Carafe',           'One litre of heatproof glass under a cork stopper.',                            'tableware',  3200, 1),
    ('rolling-pin',   'Maple Rolling Pin',      'Tapered, no handles, fifty centimetres end to end.',                            'woodwork',   3600, 1),
    ('ladle',         'Copper Ladle',           'Hammered bowl, brass rivets, a hook at the end of the handle.',                 'kitchen',    3100, 1),
    ('napkins',       'Linen Napkins',          'Four to a set, hemmed by hand, better unironed.',                               'textiles',   2400, 1),
    ('bowl',          'Porcelain Bowl',         'Thin-walled and deep enough for noodles.',                                      'tableware',  1900, 0),
    ('trivet',        'Cork Trivet',            'Twenty centimetres across, takes a pot straight off the hob.',                  'woodwork',   1500, 0),
    ('dripper',       'Pour-over Dripper',      'Glass cone on a steel stand; makes two cups at a time.',                        'kitchen',    3800, 1),
    ('tote',          'Waxed Market Tote',      'Sheds rain, carries six bottles, leather handles riveted in brass.',            'textiles',   4800, 1),
    ('butter-dish',   'Butter Dish',            'Stoneware base, beech lid, fits a whole block.',                                'tableware',  2700, 1),
    ('knife-block',   'Oak Knife Block',        'Six slots, rubber feet, solid enough to stay where you put it.',                'woodwork',   8900, 1);

-- By slug, so a row does not depend on where its product sits above.
insert into reviews (product_id, email, rating, message) values
    ((select id from products where slug = 'kettle'),      'lena@example.com',   5, 'Boils fast and the whistle is not shrill. Second one; the first was a gift.'),
    ((select id from products where slug = 'kettle'),      'marco@example.com',  4, 'The handle warms up on a gas hob. Everything else is right.'),
    ((select id from products where slug = 'skillet'),     'ines@example.com',   5, 'Eggs slide after a month of use. Mind your wrist.'),
    ((select id from products where slug = 'tea-towels'),  'piotr@example.com',  4, 'Stiff out of the box, right after three washes.'),
    ((select id from products where slug = 'board'),       'ada@example.com',    5, 'Worth the wait. Oil it and it glows.'),
    ((select id from products where slug = 'rolling-pin'), 'tomas@example.com',  3, 'Lovely wood, but I miss the handles.'),
    ((select id from products where slug = 'tote'),        'noor@example.com',   5, 'Carried a watermelon home in the rain. No complaints.');

-- Grouped by family because pagination uses IDs as cursors.
insert into materials (name, care) values
    -- metals (1-12)
    ('Enamelled steel',          'Hand wash, dry at once.'),
    ('Stainless steel',          'Dishwasher safe.'),
    ('Carbon steel',             'Season with oil; never soak.'),
    ('Cast iron',                'Season with oil; never soak.'),
    ('Copper',                   'Polish now and then; it darkens with use.'),
    ('Brass',                    'Polish now and then; it darkens with use.'),
    ('Bronze',                   'Wipe dry. The patina is the point.'),
    ('Aluminium',                'Hand wash; the dishwasher dulls it.'),
    ('Anodised aluminium',       'Dishwasher safe.'),
    ('Tin',                      'Hand wash, dry at once.'),
    ('Pewter',                   'Hand wash. Keep it away from the oven.'),
    ('Titanium',                 'Dishwasher safe.'),
    -- ceramic, glass, stone (13-24)
    ('Stoneware',                'Dishwasher and microwave safe.'),
    ('Porcelain',                'Dishwasher safe. Mind the rim.'),
    ('Earthenware',              'Hand wash. It drinks up water.'),
    ('Terracotta',               'Soak before use; never the dishwasher.'),
    ('Bone china',               'Hand wash, warm water.'),
    ('Borosilicate glass',       'Dishwasher safe. Takes the heat.'),
    ('Soda-lime glass',          'Dishwasher safe. No sudden heat.'),
    ('Tempered glass',           'Dishwasher safe.'),
    ('Marble',                   'Wipe. Oil stains; lemon etches.'),
    ('Slate',                    'Wipe, then oil once a season.'),
    ('Soapstone',                'Wipe. Oil brings the colour back.'),
    ('Granite',                  'Wipe with a damp cloth.'),
    -- wood (25-36)
    ('Walnut',                   'Oil monthly; never soak.'),
    ('Oak',                      'Oil monthly; never soak.'),
    ('Maple',                    'Oil monthly; never soak.'),
    ('Beech',                    'Wipe; oil when it looks dry.'),
    ('Ash',                      'Oil monthly; never soak.'),
    ('Cherry',                   'Oil monthly. It darkens in the light.'),
    ('Teak',                     'Wipe. It looks after itself.'),
    ('Bamboo',                   'Hand wash, stand it up to dry.'),
    ('Birch plywood',            'Wipe. Keep the edges dry.'),
    ('Acacia',                   'Oil monthly; never soak.'),
    ('Olive wood',               'Oil monthly; never soak.'),
    ('Cork',                     'Wipe with a damp cloth.'),
    -- textiles (37-48)
    ('Cotton canvas',            'Machine wash cold, line dry.'),
    ('Organic cotton',           'Machine wash cold, line dry.'),
    ('Waxed cotton',             'Brush off; re-wax once a year.'),
    ('Linen',                    'Machine wash cold. Wrinkles are fine.'),
    ('Hemp',                     'Machine wash cold, line dry.'),
    ('Jute',                     'Spot clean. Keep it dry.'),
    ('Wool',                     'Hand wash cool, dry flat.'),
    ('Merino wool',              'Hand wash cool, dry flat.'),
    ('Felt',                     'Spot clean.'),
    ('Denim',                    'Machine wash cold, inside out.'),
    ('Silk',                     'Hand wash cool, no wringing.'),
    ('Recycled polyester',       'Machine wash cold.'),
    -- other (49-60)
    ('Full-grain leather',       'Condition twice a year.'),
    ('Vegetable-tanned leather', 'Condition twice a year. It ages.'),
    ('Suede',                    'Brush; keep it out of the rain.'),
    ('Silicone',                 'Dishwasher safe.'),
    ('Natural rubber',           'Wipe. Keep it out of the sun.'),
    ('Nylon',                    'Machine wash cold.'),
    ('Melamine',                 'Dishwasher safe. Never the microwave.'),
    ('Rattan',                   'Dust; wipe with a damp cloth.'),
    ('Seagrass',                 'Dust. Keep it dry.'),
    ('Beeswax',                  'Reapply when the wood looks thirsty.'),
    ('Kraft paper',              'Recycle it.'),
    ('Concrete',                 'Seal once; wipe after.');

-- Resolve IDs by name instead of relying on fixture insertion order.
insert into product_materials (product_id, material_id)
select p.id, m.id from products p, materials m
where (p.slug, m.name) in (values
    ('kettle', 'Enamelled steel'), ('kettle', 'Stainless steel'), ('kettle', 'Beech'),
    ('apron', 'Cotton canvas'), ('apron', 'Full-grain leather'), ('apron', 'Brass'),
    ('mug', 'Stoneware'),
    ('board', 'Walnut'), ('board', 'Beeswax'),
    ('skillet', 'Cast iron'),
    ('tea-towels', 'Linen'),
    ('dinner-plate', 'Stoneware'),
    ('spoon', 'Olive wood'),
    ('pepper-mill', 'Beech'), ('pepper-mill', 'Stainless steel'),
    ('oven-mitt', 'Cotton canvas'), ('oven-mitt', 'Felt'),
    ('tumbler', 'Soda-lime glass'),
    ('salad-servers', 'Cherry'), ('salad-servers', 'Beeswax'),
    ('colander', 'Enamelled steel'),
    ('tablecloth', 'Hemp'),
    ('carafe', 'Borosilicate glass'), ('carafe', 'Cork'),
    ('rolling-pin', 'Maple'),
    ('ladle', 'Copper'), ('ladle', 'Brass'),
    ('napkins', 'Linen'),
    ('bowl', 'Porcelain'),
    ('trivet', 'Cork'),
    ('dripper', 'Borosilicate glass'), ('dripper', 'Stainless steel'),
    ('tote', 'Waxed cotton'), ('tote', 'Vegetable-tanned leather'), ('tote', 'Brass'),
    ('butter-dish', 'Stoneware'), ('butter-dish', 'Beech'),
    ('knife-block', 'Oak'), ('knife-block', 'Natural rubber'));
"#;
