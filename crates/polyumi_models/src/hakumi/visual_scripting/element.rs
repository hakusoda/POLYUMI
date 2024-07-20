use serde::{ Deserialize, Serialize };
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Serialize, Validate)]
pub struct ElementModel {
	pub id: Uuid,

	#[serde(flatten)]
	pub kind: ElementKind
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum ElementKind {
	#[serde(rename = "action.mellow.member.ban")]
	BanMember(VariableReference),
	#[serde(rename = "action.mellow.member.kick")]
	KickMember(VariableReference),
	#[serde(rename = "action.mellow.member.sync")]
	SyncMember,

	#[serde(rename = "action.mellow.member.roles.assign")]
	AssignRoleToMember(StringValueWithVariableReference),
	#[serde(rename = "action.mellow.member.roles.remove")]
	RemoveRoleFromMember(StringValueWithVariableReference),

	#[serde(rename = "action.mellow.message.reply")]
	Reply(StringValueWithVariableReference),
	#[serde(rename = "action.mellow.message.reaction.create")]
	AddReaction(StringValueWithVariableReference),

	#[serde(rename = "action.mellow.message.create")]
	CreateMessage(Message),
	#[serde(rename = "action.mellow.message.delete")]
	DeleteMessage(VariableReference),

	#[serde(rename = "action.mellow.message.start_thread")]
	StartThreadFromMessage {
		name: Text,
		message: VariableReference
	},

	#[serde(rename = "action.mellow.interaction.reply")]
	InteractionReply(Text),

	#[serde(rename = "get_data.mellow.server.current_patreon_campaign")]
	GetLinkedPatreonCampaign,

	#[serde(rename = "no_op.comment")]
	Comment,
	#[serde(rename = "no_op.nothing")]
	Nothing,

	#[serde(rename = "special.root")]
	Root,

	#[serde(rename = "statement.if")]
	IfStatement(ConditionalStatement)
}

#[derive(Deserialize, Serialize, Validate)]
pub struct VariableReference {
	#[validate(length(max = 128))]
	path: String
}

#[derive(Deserialize, Serialize)]
pub struct StringValueWithVariableReference {
	pub value: String,
	pub reference: VariableReference
}

#[derive(Deserialize, Serialize)]
pub struct Message {
	pub content: Text,
	pub channel_id: StatementInput
}

#[derive(Deserialize, Serialize, Validate)]
pub struct Text {
	#[validate(length(max = 32))]
	pub value: Vec<TextElement>
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum TextElement {
	String(String),
	Variable(VariableReference)
}

#[derive(Deserialize, Serialize, Validate)]
pub struct ConditionalStatement {
	#[validate(length(max = 16))]
	pub blocks: Vec<StatementBlock>
}

#[derive(Deserialize, Serialize, Validate)]
pub struct StatementBlock {
	#[validate(length(max = 16))]
	pub items: Vec<ElementModel>,

	#[validate(length(max = 8))]
	pub conditions: Vec<StatementCondition>
}

#[derive(Deserialize, Serialize, Validate)]
pub struct StatementCondition {
	pub kind: StatementConditionKind,
	#[validate(length(max = 8))]
	pub inputs: Vec<StatementInput>,
	pub condition: Condition
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StatementConditionKind {
	Initial,
	And,
	Or
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum Condition {
	#[serde(rename = "generic.is")]
	Is,
	#[serde(rename = "generic.is_not")]
	IsNot,

	#[serde(rename = "iterable.has_any_value")]
	HasAnyValue,
	#[serde(rename = "iterable.does_not_have_any_value")]
	DoesNotHaveAnyValue,
	#[serde(rename = "iterable.contains")]
	Contains,
	#[serde(rename = "iterable.contains_only")]
	ContainsOnly,
	#[serde(rename = "iterable.contains_one_of")]
	ContainsOneOf,
	#[serde(rename = "iterable.does_not_contain")]
	DoesNotContain,
	#[serde(rename = "iterable.does_not_contain_one_of")]
	DoesNotContainOneOf,
	#[serde(rename = "iterable.begins_with")]
	BeginsWith,
	#[serde(rename = "iterable.ends_with")]
	EndsWith
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum StatementInput {
	Match(serde_json::Value),
	Variable(VariableReference)
}