// @generated automatically by Diesel CLI.

diesel::table! {
    discorduser (did) {
        did -> Unsigned<Bigint>,
        #[max_length = 25]
        username -> Varchar,
    }
}

diesel::table! {
    twitchuser (tid) {
        tid -> Unsigned<Integer>,
        #[max_length = 25]
        username -> Varchar,
    }
}

diesel::table! {
    users (id) {
        id -> Unsigned<Integer>,
        uid -> Unsigned<Integer>,
        discord_id -> Unsigned<Bigint>,
        twitch_id -> Unsigned<Integer>,
    }
}

diesel::joinable!(users -> discorduser (discord_id));
diesel::joinable!(users -> twitchuser (twitch_id));

diesel::allow_tables_to_appear_in_same_query!(discorduser, twitchuser, users,);
