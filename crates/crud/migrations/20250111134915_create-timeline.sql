
--------------------------------------------------------------------------------
-- Data
--------------------------------------------------------------------------------

CREATE TABLE entities (
    id                 TEXT NOT NULL UNIQUE,
    name               TEXT NOT NULL UNIQUE,
    start_year         SMALLINT NOT NULL,
    start_month        TINYINT UNSIGNED,
    start_day          TINYINT UNSIGNED,
    end_year           SMALLINT,
    end_month          TINYINT UNSIGNED,
    end_day            TINYINT UNSIGNED,

    PRIMARY KEY (id)
);

CREATE TABLE entity_tags (
    entity_id          TEXT NOT NULL,
    name               TEXT,
    value              TEXT NOT NULL,

    FOREIGN KEY (entity_id) REFERENCES entities (id)
);

CREATE TABLE timelines (
    id                 TEXT NOT NULL UNIQUE,
    name               TEXT NOT NULL UNIQUE,
    bool_expression    TEXT,

    PRIMARY KEY (id)
);

CREATE TABLE subtimelines (
    timeline_parent_id TEXT NOT NULL,
    timeline_child_id  TEXT NOT NULL,

    FOREIGN KEY (timeline_parent_id) REFERENCES timelines (id),
    FOREIGN KEY (timeline_child_id)  REFERENCES timelines (id)
);

CREATE TABLE timeline_entities (
    timeline_id        TEXT NOT NULL,
    entity_id          TEXT NOT NULL,

    FOREIGN KEY (timeline_id) REFERENCES timelines (id),
    FOREIGN KEY (entity_id)   REFERENCES entities (id)
);

CREATE TABLE timeline_tags (
    timeline_id        TEXT NOT NULL,
    name               TEXT,
    value              TEXT NOT NULL,

    FOREIGN KEY (timeline_id) REFERENCES timelines (id)
);

--------------------------------------------------------------------------------
-- Indexes
--------------------------------------------------------------------------------

-- entities table
CREATE INDEX idx_entities_id
    ON entities(id);
CREATE INDEX idx_entities_start_year
    ON entities(start_year);
CREATE INDEX idx_entities_end_year
    ON entities(end_year);

--- entity_tags table
CREATE INDEX idx_entity_tags_entity_id
    ON entity_tags(entity_id);
CREATE INDEX idx_entity_tags_name
    ON entity_tags(name);
CREATE INDEX idx_entity_tags_value
    ON entity_tags(value);

--- timelines table
CREATE INDEX idx_timelines_id
    ON timelines(id);

--- subtimelines table
CREATE INDEX idx_subtimelines_timeline_parent_id
    ON subtimelines(timeline_parent_id);
CREATE INDEX idx_subtimelines_timeline_child_id
    ON subtimelines(timeline_child_id);

--- timeline_entities table
CREATE INDEX idx_timeline_entities_timeline_id
    ON timeline_entities(timeline_id);
CREATE INDEX idx_timeline_entities_entity_id
    ON timeline_entities(entity_id);

--- timeline_tags table
CREATE INDEX idx_timeline_tags_timeline_id
    ON timeline_tags(timeline_id);
CREATE INDEX idx_timeline_tags_name
    ON timeline_tags(name);
CREATE INDEX idx_timeline_tags_value
    ON timeline_tags(value);
