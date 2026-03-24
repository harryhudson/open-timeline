// Entities with a start year after 1600
export async function get_entities() {
	const entities = await all_entities();
	const filtered = entities.filter(entity => entity.start.year > 1600);
	return filtered.slice(0, 4)
}

// Read all entities from entities.json
export async function all_entities() {
	const res = await fetch('../other/entities.json');
	return await res.json();
}
