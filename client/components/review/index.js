const React = require('react');
const Immutable = require('immutable');
const either = require('react-either');
const invariant = require('invariant');

const {types: ROUTES} = require('store/routes');
const {ReviewPatch} = require('store/review');

const courier = require('courier');

const {tabs} = require('constants/cardprofile');

const CardDetail = require('components/card/index');
const ReviewTabBar = require('./tabbar');
const difficulty = require('constants/difficulty');

const Review = React.createClass({

    contextTypes: {
        store: React.PropTypes.object.isRequired
    },

    propTypes: {
        currentDeckID: React.PropTypes.number.isRequired,
        currentTab: React.PropTypes.oneOf([tabs.front, tabs.back, tabs.description, tabs.stashes, tabs.meta]),
        card: React.PropTypes.instanceOf(Immutable.Map).isRequired
    },

    getInitialState() {
        return {

            isEditing: false,
            disableSave: false,

            difficulty: difficulty.none,

            // hide back-side of card when being reviewed
            reveal: false
        };
    },

    resetState() {

        this.setState({

            isEditing: false,
            disableSave: false,

            difficulty: difficulty.none,

            // hide back-side of card when being reviewed
            reveal: false
        });

        const deckID = this.props.currentDeckID;
        this.context.store.routes.toDeckReviewCardFront(deckID);
    },

    componentWillMount() {

        // redirect to front side of card if back-side shouldn't be revealed yet
        if(this.props.currentTab === tabs.back && !this.state.reveal) {

            const deckID = this.props.currentDeckID;
            this.context.store.routes.toDeckReviewCardFront(deckID);
        }

    },

    onClickBackButton() {
        this.context.store.routes.toLibraryCards();
    },

    onSwitchCurrentTab(tabType) {

        const deckID = this.props.currentDeckID;

        switch(tabType) {

        case tabs.front:

            this.context.store.routes.toDeckReviewCardFront(deckID);
            break;

        case tabs.back:

            this.context.store.routes.toDeckReviewCardBack(deckID);
            break;

        case tabs.description:

            this.context.store.routes.toDeckReviewCardDescription(deckID);
            break;

        case tabs.meta:

            this.context.store.routes.toDeckReviewCardMeta(deckID);
            break;

        case tabs.stashes:

            this.context.store.routes.toDeckReviewCardStashes(deckID);
            break;

        default:
            invariant(false, `Unexpected tabType. Given: ${String(tabType)}`);

        }
    },

    onCardSave(patch) {

        this.setState({
            isEditing: false
        });

        const cardID = this.props.card.get('id');

        this.context.store.cards.patch(cardID, patch);
    },

    editCard() {

        this.setState({
            isEditing: true
        });

    },

    onCancelEdit() {

        this.setState({
            isEditing: false
        });

    },

    onDelete() {

        const deckID = this.props.currentDeckID;
        this.context.store.routes.toLibraryCards(deckID);
    },

    onReveal() {

        const deckID = this.props.currentDeckID;

        this.context.store.routes.toDeckReviewCardBack(deckID);

        this.setState({
            reveal: true
        });

    },

    onNext() {

        const {currentDeckID} = this.props;

        const cardID = this.props.card.get('id');

        const currentDifficulty = this.state.difficulty;

        const patch = new ReviewPatch(cardID);

        patch.difficulty(currentDifficulty);
        patch.skipCard(false);
        patch.deck(currentDeckID);

        this.context.store.review.reviewCard(patch)
            .then(() => {

                this.resetState();

                return this.context.store.review.getNextReviewableCardForDeck();
            });

    },

    onSkip() {

        const {currentDeckID} = this.props;

        const cardID = this.props.card.get('id');

        const currentDifficulty = this.state.difficulty;

        const patch = new ReviewPatch(cardID);

        patch.difficulty(currentDifficulty);
        patch.skipCard(true);
        patch.deck(currentDeckID);

        this.context.store.review.reviewCard(patch)
            .then(() => {

                this.resetState();

                return this.context.store.review.getNextReviewableCardForDeck();
            });

    },

    onChooseDifficulty(difficultyTag) {

        this.setState({
            difficulty: difficultyTag
        });
    },

    render() {

        // bail early
        if(this.props.currentTab === tabs.back && !this.state.reveal) {
            return null;
        }

        return (
            <div>
                <div className="row">
                    <div className="col-sm-12">
                        <CardDetail

                            isReviewing
                            hideBack={!this.state.reveal}
                            hideEdit={!this.state.reveal}

                            currentTab={this.props.currentTab}
                            currentCard={this.props.card}

                            isEditing={this.state.isEditing}
                            disableSave={this.state.disableSave}

                            backButtonLabel="Stop Reviewing Deck"
                            onClickBackButton={this.onClickBackButton}

                            onSwitchCurrentTab={this.onSwitchCurrentTab}

                            onCardSave={this.onCardSave}
                            editCard={this.editCard}
                            onCancelEdit={this.onCancelEdit}
                            onDelete={this.onDelete}
                        />
                    </div>
                </div>
                <div className="row">
                    <div className="col-sm-12">
                        <hr />
                    </div>
                </div>
                <div className="row">
                    <div className="col-sm-12">
                        <ReviewTabBar
                            reveal={this.state.reveal}
                            difficulty={this.state.difficulty}
                            onReveal={this.onReveal}
                            onNext={this.onNext}
                            onSkip={this.onSkip}
                            onChooseDifficulty={this.onChooseDifficulty}
                        />
                    </div>
                </div>
            </div>
        );

    }
});

const NoReview = React.createClass({

    contextTypes: {
        store: React.PropTypes.object.isRequired
    },

    backToCardsList(event) {
        event.preventDefault();
        event.stopPropagation();

        this.context.store.routes.toLibraryCards();
    },

    render() {
        return (
            <div>
                <div className="row">
                    <div className="col-sm-12 m-y">
                        <button
                            type="button"
                            className="btn btn-sm btn-danger"
                            onClick={this.backToCardsList}
                        >
                            {'Stop Reviewing Deck'}
                        </button>
                    </div>
                </div>
                <div className="row">
                    <div className="col-sm-12">
                        <div className="card">
                            <div className="card-block text-center">
                                <p className="card-text text-muted">
                                    {'This deck does not have any cards for review. Add or create new cards for this deck.'}
                                </p>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        );
    }
});

const eitherReview = either(Review, NoReview, function(props) {

    // if props.card is not Immutable.Map, then there is no reviewable card for this deck

    return Immutable.Map.isMap(props.card);
});

module.exports = courier({

    contextTypes: {
        store: React.PropTypes.object.isRequired
    },

    component: eitherReview,
    onlyWaitingOnMount: true,

    watch(props, manual, context) {
        return [
            context.store.routes.watchRoute(),
            context.store.review.watchCardOfCurrentDeck()
        ];
    },

    assignNewProps: function(props, context) {

        // console.log(context.store.routes.route());

        // fetch reviewable card for deck
        return context.store.review.getReviewableCardForDeck()
            .then(function(card) {

                const route = context.store.routes.route();

                let currentTab;

                switch(route) {

                case ROUTES.REVIEW.VIEW.FRONT:

                    currentTab = tabs.front;
                    break;

                case ROUTES.REVIEW.VIEW.BACK:

                    currentTab = tabs.back;
                    break;

                case ROUTES.REVIEW.VIEW.DESCRIPTION:

                    currentTab = tabs.description;
                    break;

                case ROUTES.REVIEW.VIEW.META:

                    currentTab = tabs.meta;
                    break;

                case ROUTES.REVIEW.VIEW.STASHES:

                    currentTab = tabs.stashes;
                    break;

                default:
                    invariant(false, `Unexpected route. Given: ${String(this.props.route)}`);
                }

                const currentDeckID = context.store.decks.currentID();

                return {
                    card: card,
                    currentTab: currentTab,
                    currentDeckID: currentDeckID
                };

            });

    }

});
