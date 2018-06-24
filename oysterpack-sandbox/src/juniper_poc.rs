// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use juniper;
use juniper::FieldResult;

#[derive(GraphQLEnum)]
enum Episode {
    NewHope,
    Empire,
    Jedi,
}

#[derive(GraphQLObject)]
#[graphql(description = "A humanoid creature in the Star Wars universe")]
struct Human {
    id: String,
    name: String,
    appears_in: Vec<Episode>,
    home_planet: String,
}

#[derive(GraphQLInputObject)]
#[graphql(description = "A humanoid creature in the Star Wars universe")]
struct NewHuman {
    name: String,
    appears_in: Vec<Episode>,
    home_planet: String,
}

struct DatabasePool;

impl DatabasePool {
    fn get_connection(&self) -> Result<Connection, String> {
        Ok(Connection)
    }
}

struct Connection;

impl Connection {
    fn find_human(&self, id: &str) -> Result<Human, String> {
        Ok(Human {
            id: id.to_string(),
            name: "Luke".to_string(),
            appears_in: vec![],
            home_planet: "Earth".to_string(),
        })
    }

    fn insert_human(&self, new_human: &NewHuman) -> Result<Human, String> {
        Ok(Human {
            id: "123".to_string(),
            name: new_human.name.clone(),
            appears_in: vec![],
            home_planet: new_human.home_planet.clone(),
        })
    }
}

struct Context {
    // Use your real database pool here.
    pool: DatabasePool,
}

// To make our context usable by Juniper, we have to implement a marker trait.
impl juniper::Context for Context {}

struct Query;

graphql_object!(Query: Context |&self| {

    field apiVersion() -> &str {
        "1.0"
    }

    // Arguments to resolvers can either be simple types or input objects.
    // The executor is a special (optional) argument that allows accessing the context.
    field human(&executor, id: String) -> FieldResult<Human> {
        // Get the context from the executor.
        let context = executor.context();
        // Get a db connection.
        let connection = context.pool.get_connection()?;
        // Execute a db query.
        // Note the use of `?` to propagate errors.
        let human = connection.find_human(&id)?;
        // Return the result.
        Ok(human)
    }
});

struct Mutation;

graphql_object!(Mutation: Context |&self| {

    field createHuman(&executor, new_human: NewHuman) -> FieldResult<Human> {
        let db = executor.context().pool.get_connection()?;
        let human: Human = db.insert_human(&new_human)?;
        Ok(human)
    }
});

// A root schema consists of a query and a mutation.
// Request queries can be executed against a RootNode.
type Schema = juniper::RootNode<'static, Query, Mutation>;
