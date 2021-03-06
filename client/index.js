const React = require('react');
const ReactDOM = require('react-dom');
// const co = require('co');

// superagent proxy
const superhot = require('store/superhot');
const littleloader = require('little-loader');

const bootstrapStore = require('store');
const App = require('components/app');

const isSymbol = require('./utils/issymbol');


const loadMathJax = new Promise(function(resolve) {

    // check if mathjax assets exist

    superhot.get('/mathjax/MathJax.js').end(function(err, response){
        resolve(response.status == 200);
    });

}).then(function(hasLocalMathJax) {

    const mjscript = hasLocalMathJax ? '/mathjax/MathJax.js?config=TeX-AMS-MML_HTMLorMML' :
        'https://cdn.mathjax.org/mathjax/latest/MathJax.js?config=TeX-AMS-MML_HTMLorMML';

    return new Promise(function(resolve) {

        littleloader(mjscript, function (err) {

            if(err) {
                console.error(err);
            }

            resolve(null);
        });

    });
});

const loadStore = bootstrapStore
    .then(function(store) {

        return new Promise(function(resolve) {
            store.resetStage();

            const route = store.routes.route();
            if(isSymbol(route)) {
                resolve(store);
                return;
            }

            const cursor = store.state().cursor('route');

            const waiting = function() {

                cursor.once('any', function(_route) {

                    if(isSymbol(_route)) {
                        resolve(store);
                        return;
                    }

                    waiting();
                });
            };

            waiting();

        });
    });

Promise.all([loadStore, loadMathJax])
    .then(function(results) {
        return Promise.resolve(results[0]);
    })
    .then(function(store) {

        // TODO: this is temporary fix; remediate this later
        try {
            console.log('====');
            console.log(String(store.state()));
            console.log('====');
        } catch(err) {
            console.error(err);
        }

        // As of React v0.14.x, contexts are experimental.
        // see: https://facebook.github.io/react/docs/context.html
        //
        // Use context to infect any and all component with the reference to the store.
        // This infection is stateless, and any component that uses it need only be
        // infected once.
        const WithContext = React.createClass({

            childContextTypes: {
                store: React.PropTypes.object.isRequired
            },

            getChildContext: function() {
                return {
                    store: store
                };
            },

            render: function() {
                return (<App {...this.props} />);
            }
        });

        ReactDOM.render(<WithContext />, document.getElementById('grokdb-container'));

        // silence bluebird v3 warnings
        return null;
    });

// TODO: issue: https://github.com/petkaantonov/bluebird/issues/903
// co(function* () {

//     const store = yield bootstrapStore;

//     // ensure and wait for route to be settled

//     yield new Promise(function(resolve) {
//         store.resetStage();

//         const route = store.routes.route();
//         if(isSymbol(route)) {
//             resolve();
//         }

//         const cursor = store.state().cursor('route');

//         const waiting = function() {

//             cursor.once('any', function(_route) {

//                 if(isSymbol(_route)) {
//                     resolve();
//                     return;
//                 }

//                 waiting();
//             });
//         };

//         waiting();

//     });

//     console.log('====');
//     console.log(String(store.state()));
//     console.log('====');

//     // NOTE: As of react v0.13, contexts are an undocumented feature
//     // NOTE: As of react v0.13, React.withContext() is deprecated.
//     // See: https://www.tildedave.com/2014/11/15/introduction-to-contexts-in-react-js.html
//     const WithContext = React.createClass({

//         childContextTypes: {
//             store: React.PropTypes.object.isRequired
//         },

//         getChildContext: function() {
//             return {
//                 store: store
//             };
//         },

//         render: function() {
//             return (<App {...this.props} />);
//         }
//     });

//     ReactDOM.render(<WithContext rootCursor={store.state()} />, document.getElementById('grokdb-container'));

// });
