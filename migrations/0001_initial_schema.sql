PRAGMA foreign_keys = ON;

CREATE TABLE vocabulary (
    id INTEGER PRIMARY KEY,
    expression TEXT NOT NULL,
    reading TEXT,
    meaning TEXT NOT NULL,
    part_of_speech TEXT,
    jlpt_level TEXT,
    frequency_rank INTEGER,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE kanji (
    id INTEGER PRIMARY KEY,
    character TEXT NOT NULL UNIQUE,
    meaning TEXT NOT NULL,
    on_readings TEXT,
    kun_readings TEXT,
    stroke_count INTEGER,
    jlpt_level TEXT,
    grade INTEGER,
    notes TEXT
);

CREATE TABLE grammar (
    id INTEGER PRIMARY KEY,
    pattern TEXT NOT NULL,
    meaning TEXT NOT NULL,
    jlpt_level TEXT,
    explanation TEXT,
    notes TEXT
);

CREATE TABLE examples (
    id INTEGER PRIMARY KEY,
    japanese TEXT NOT NULL,
    translation TEXT,
    source_type TEXT
);

CREATE TABLE vocabulary_examples (
    vocabulary_id INTEGER NOT NULL REFERENCES vocabulary(id) ON DELETE CASCADE,
    example_id INTEGER NOT NULL REFERENCES examples(id) ON DELETE CASCADE,
    PRIMARY KEY (vocabulary_id, example_id)
);

CREATE TABLE kanji_vocabulary (
    kanji_id INTEGER NOT NULL REFERENCES kanji(id) ON DELETE CASCADE,
    vocabulary_id INTEGER NOT NULL REFERENCES vocabulary(id) ON DELETE CASCADE,
    PRIMARY KEY (kanji_id, vocabulary_id)
);

CREATE TABLE user_vocabulary (
    vocabulary_id INTEGER PRIMARY KEY REFERENCES vocabulary(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'new',
    ease REAL NOT NULL DEFAULT 2.5,
    interval_days INTEGER NOT NULL DEFAULT 0,
    repetitions INTEGER NOT NULL DEFAULT 0,
    lapses INTEGER NOT NULL DEFAULT 0,
    due_at TEXT,
    last_reviewed_at TEXT
);

CREATE TABLE user_kanji (
    kanji_id INTEGER PRIMARY KEY REFERENCES kanji(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'new',
    ease REAL NOT NULL DEFAULT 2.5,
    interval_days INTEGER NOT NULL DEFAULT 0,
    repetitions INTEGER NOT NULL DEFAULT 0,
    lapses INTEGER NOT NULL DEFAULT 0,
    due_at TEXT,
    last_reviewed_at TEXT
);

CREATE TABLE reviews (
    id INTEGER PRIMARY KEY,
    item_type TEXT NOT NULL,
    item_id INTEGER NOT NULL,
    direction TEXT NOT NULL,
    rating INTEGER NOT NULL CHECK (rating BETWEEN 1 AND 4),
    response_ms INTEGER,
    reviewed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX vocabulary_expression_idx ON vocabulary(expression);
CREATE INDEX vocabulary_reading_idx ON vocabulary(reading);
CREATE INDEX vocabulary_jlpt_level_idx ON vocabulary(jlpt_level);
CREATE INDEX kanji_jlpt_level_idx ON kanji(jlpt_level);
CREATE INDEX grammar_jlpt_level_idx ON grammar(jlpt_level);
CREATE INDEX user_vocabulary_due_at_idx ON user_vocabulary(due_at) WHERE due_at IS NOT NULL;
CREATE INDEX user_kanji_due_at_idx ON user_kanji(due_at) WHERE due_at IS NOT NULL;
CREATE INDEX reviews_item_idx ON reviews(item_type, item_id, reviewed_at DESC);
CREATE INDEX reviews_reviewed_at_idx ON reviews(reviewed_at DESC);
